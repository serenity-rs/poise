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

    let guild = match ctx.cache.guild(guild_id) {
        Some(x) => x,
        None => return None, // Guild not in cache
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

    // If member not in cache (probably because presences intent is not enabled), retrieve via HTTP
    let member = match guild.members.get(&user_id) {
        Some(x) => x.clone(),
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
#[allow(clippy::needless_lifetimes)] // false positive (clippy issue 7271)
pub async fn check_permissions_and_cooldown<'a, U, E>(
    ctx: crate::Context<'a, U, E>,
    cmd: &crate::Command<U, E>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    if cmd.owners_only && !ctx.framework().options().owners.contains(&ctx.author().id) {
        return Err(crate::FrameworkError::NotAnOwner { ctx });
    }

    if cmd.guild_only && ctx.guild_id().is_none() {
        return Err(crate::FrameworkError::GuildOnly { ctx });
    }

    if cmd.dm_only && ctx.guild_id().is_some() {
        return Err(crate::FrameworkError::DmOnly { ctx });
    }

    if cmd.nsfw_only {
        match ctx
            .channel_id()
            .to_channel(ctx.discord())
            .await
            .unwrap()
            .is_nsfw()
        {
            true => (),
            false => return Err(crate::FrameworkError::NsfwOnly { ctx }),
        };
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
    let bot_user_id = ctx.discord().cache.current_user_id();
    match missing_permissions(ctx, bot_user_id, cmd.required_bot_permissions).await {
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
        cooldowns.lock().unwrap().start_cooldown(ctx);
    }

    Ok(())
}
