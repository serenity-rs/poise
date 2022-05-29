//! Prefix and slash agnostic utilities for dispatching incoming events onto framework commands

use crate::serenity_prelude as serenity;

/// Retrieves user permissions in the given channel. If unknown, returns None. If in DMs, returns
/// `Permissions::all()`.
async fn user_permissions(
    ctx: &serenity::Context,
    guild_id: Option<serenity::GuildId>,
    channel_id: serenity::ChannelId,
    user_id: serenity::UserId,
) -> Option<serenity::Permissions> {
    let guild_id = match guild_id {
        Some(x) => x,
        None => return Some(serenity::Permissions::all()), // no permission checks in DMs
    };

    #[cfg(feature = "cache")]
    let guild = match ctx.cache.guild(guild_id) {
        Some(x) => x,
        None => return None, // Guild not in cache
    };
    #[cfg(not(feature = "cache"))]
    let guild = match ctx.http.get_guild(guild_id.0).await {
        Ok(x) => x,
        Err(_) => return None,
    };

    // Use to_channel so that it can fallback on HTTP for threads (which aren't in cache usually)
    let channel = match channel_id.to_channel(ctx).await {
        Ok(serenity::Channel::Guild(channel)) => channel,
        Ok(_other_channel) => {
            log::warn!(
                "guild message was supposedly sent in a non-guild channel. Denying invocation"
            );
            return None;
        }
        Err(_) => return None,
    };

    #[cfg(feature = "cache")]
    let cached_member = guild.members.get(&user_id).cloned();
    #[cfg(not(feature = "cache"))]
    let cached_member = None;

    // If member not in cache (probably because presences intent is not enabled), retrieve via HTTP
    let member = match cached_member {
        Some(x) => x,
        None => match ctx.http.get_member(guild_id.0, user_id.0).await {
            Ok(member) => member,
            Err(_) => return None,
        },
    };

    guild.user_permissions_in(&channel, &member).ok()
}

/// Retrieves the set of permissions that are lacking, relative to the given required permission set
///
/// Returns None if permissions couldn't be retrieved
async fn missing_permissions<U, E>(
    ctx: crate::Context<'_, U, E>,
    user: serenity::UserId,
    required_permissions: serenity::Permissions,
) -> Option<serenity::Permissions> {
    if required_permissions.is_empty() {
        return Some(serenity::Permissions::empty());
    }

    let permissions = user_permissions(ctx.discord(), ctx.guild_id(), ctx.channel_id(), user).await;
    Some(required_permissions - permissions?)
}

/// Checks if the invoker is allowed to execute this command at this point in time
///
/// Doesn't actually start the cooldown timer! This should be done by the caller later, after
/// argument parsing.
/// (A command that didn't even get past argument parsing shouldn't trigger cooldowns)
#[allow(clippy::needless_lifetimes)] // false positive (clippy issue 7271)
pub async fn check_permissions_and_cooldown<'a, U, E>(
    ctx: crate::Context<'a, U, E>,
    cmd: &crate::Command<U, E>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    if cmd.owners_only && !ctx.framework().options().owners.contains(&ctx.author().id) {
        return Err(crate::FrameworkError::NotAnOwner { ctx });
    }

    if cmd.guild_only {
        match ctx.guild_id() {
            None => return Err(crate::FrameworkError::GuildOnly { ctx }),
            Some(guild_id) => {
                #[cfg(feature = "cache")]
                if ctx.framework().options().require_cache_for_guild_check
                    && ctx.discord().cache.guild_field(guild_id, |_| ()).is_none()
                {
                    return Err(crate::FrameworkError::GuildOnly { ctx });
                }
                #[cfg(not(feature = "cache"))]
                let _ = guild_id;
            }
        }
    }

    if cmd.dm_only && ctx.guild_id().is_some() {
        return Err(crate::FrameworkError::DmOnly { ctx });
    }

    if cmd.nsfw_only {
        let channel = match ctx.channel_id().to_channel(ctx.discord()).await {
            Ok(channel) => channel,
            Err(e) => {
                log::warn!("Error when getting channel: {}", e);

                return Err(crate::FrameworkError::NsfwOnly { ctx });
            }
        };

        if !channel.is_nsfw() {
            return Err(crate::FrameworkError::NsfwOnly { ctx });
        }
    }

    // Make sure that user has required permissions
    match missing_permissions(ctx, ctx.author().id, cmd.required_permissions).await {
        Some(missing_permissions) if missing_permissions.is_empty() => {}
        Some(missing_permissions) => {
            return Err(crate::FrameworkError::MissingUserPermissions {
                ctx,
                missing_permissions: Some(missing_permissions),
            })
        }
        // Better safe than sorry: when perms are unknown, restrict access
        None => {
            return Err(crate::FrameworkError::MissingUserPermissions {
                ctx,
                missing_permissions: None,
            })
        }
    }

    // Before running any pre-command checks, make sure the bot has the permissions it needs
    match missing_permissions(ctx, ctx.framework().bot_id, cmd.required_bot_permissions).await {
        Some(missing_permissions) if missing_permissions.is_empty() => {}
        Some(missing_permissions) => {
            return Err(crate::FrameworkError::MissingBotPermissions {
                ctx,
                missing_permissions,
            })
        }
        // When in doubt, just let it run. Not getting fancy missing permissions errors is better
        // than the command not executing at all
        None => {}
    }

    // Only continue if command checks returns true. First perform global checks, then command
    // checks (if necessary)
    for check in [ctx.framework().options().command_check, cmd.check]
        .iter()
        .flatten()
    {
        match check(ctx).await {
            Ok(true) => {}
            Ok(false) => {
                return Err(crate::FrameworkError::CommandCheckFailed { ctx, error: None })
            }
            Err(error) => {
                return Err(crate::FrameworkError::CommandCheckFailed {
                    error: Some(error),
                    ctx,
                })
            }
        }
    }

    if !ctx.framework().options().manual_cooldowns {
        let cooldowns = &cmd.cooldowns;
        let remaining_cooldown = cooldowns.lock().unwrap().remaining_cooldown(ctx);
        if let Some(remaining_cooldown) = remaining_cooldown {
            return Err(crate::FrameworkError::CooldownHit {
                ctx,
                remaining_cooldown,
            });
        }
    }

    Ok(())
}

/// Should be invoked after running a command. As long as the command didn't fail due to argument
/// parsing, this function will trigger the cooldown counter
pub fn trigger_cooldown_maybe<U, E>(
    ctx: crate::Context<'_, U, E>,
    res: &Result<(), crate::FrameworkError<'_, U, E>>,
) {
    if !ctx.framework().options.manual_cooldowns {
        if let Err(crate::FrameworkError::ArgumentParse { .. }) = res {
            // Argument parse errors shouldn't count towards cooldown
        } else {
            ctx.command().cooldowns.lock().unwrap().start_cooldown(ctx);
        }
    }
}
