//! Prefix and slash agnostic utilities for dispatching incoming events onto framework commands

use crate::serenity_prelude as serenity;

/// Retrieves user permissions in the given channel. If unknown, returns None. If in DMs, returns
/// `Permissions::all()`.
async fn user_permissions<U, E>(
    ctx: crate::Context<'_, U, E>,
    user_id: Option<serenity::UserId>,
    bot_id: Option<serenity::UserId>,
) -> Option<(Option<serenity::Permissions>, Option<serenity::Permissions>)> {
    let guild_id = ctx.guild_id();
    let channel_id = ctx.channel_id();
    let guild_id = match guild_id {
        Some(x) => x,
        // no permission checks in DMs
        None => {
            return Some((
                Some(serenity::Permissions::all()),
                Some(serenity::Permissions::all()),
            ))
        }
    };

    match ctx {
        crate::Context::Application(ctx) => {
            // This should be present on all interactions within a guild. But discord can be a bit
            // funny sometimes, so lets be safe.
            if let Some(member) = &ctx.interaction.member {
                return Some((member.permissions, ctx.interaction.app_permissions));
            }
        }
        crate::Context::Prefix(_) => {}
    }

    // Use to_channel so that it can fallback on HTTP for threads (which aren't in cache usually)
    let channel = match channel_id.to_channel(ctx.serenity_context()).await {
        Ok(serenity::Channel::Guild(channel)) => channel,
        Ok(_other_channel) => {
            tracing::warn!(
                "guild message was supposedly sent in a non-guild channel. Denying invocation"
            );
            return None;
        }
        Err(_) => return None,
    };

    // These are done by HTTP only to prevent outdated data with no GUILD_MEMBERS intent.
    let user_member = if let Some(user_id) = user_id {
        Some(
            guild_id
                .member(ctx.serenity_context(), user_id)
                .await
                .ok()?,
        )
    } else {
        None
    };

    let bot_member = if let Some(bot_id) = bot_id {
        Some(guild_id.member(ctx.serenity_context(), bot_id).await.ok()?)
    } else {
        None
    };

    #[cfg(feature = "cache")]
    let cached_perms = {
        ctx.cache().guild(guild_id).map(|guild| {
            (
                user_member
                    .as_ref()
                    .map(|m| guild.user_permissions_in(&channel, m)),
                bot_member
                    .as_ref()
                    .map(|m| guild.user_permissions_in(&channel, m)),
            )
        })
    };

    #[cfg(feature = "cache")]
    let users_permissions = if let Some(users_permissions) = cached_perms {
        users_permissions
    } else {
        let partial_guild = guild_id
            .to_partial_guild(ctx.serenity_context())
            .await
            .ok()?;
        (
            user_member.map(|m| partial_guild.user_permissions_in(&channel, &m)),
            bot_member.map(|m| partial_guild.user_permissions_in(&channel, &m)),
        )
    };

    #[cfg(not(feature = "cache"))]
    let partial_guild = guild_id
        .to_partial_guild(ctx.serenity_context())
        .await
        .ok()?;
    #[cfg(not(feature = "cache"))]
    let users_permissions = (
        user_member.map(|m| partial_guild.user_permissions_in(&channel, &m)),
        bot_member.map(|m| partial_guild.user_permissions_in(&channel, &m)),
    );

    Some(users_permissions)
}

/// Retrieves the set of permissions that are lacking, relative to the given required permission set
///
/// Returns None if permissions couldn't be retrieved.
async fn missing_permissions<U, E>(
    ctx: crate::Context<'_, U, E>,
    user: (serenity::UserId, serenity::Permissions),
    bot: (serenity::UserId, serenity::Permissions),
) -> Option<(serenity::Permissions, serenity::Permissions)> {
    // If both user and bot are None, return empty permissions
    if user.1.is_empty() && bot.1.is_empty() {
        return Some((
            serenity::Permissions::empty(),
            serenity::Permissions::empty(),
        ));
    }

    let user_id = match user.1.is_empty() {
        true => None,
        false => Some(user.0),
    };

    let bot_id = match bot.1.is_empty() {
        true => None,
        false => Some(bot.0),
    };

    // Fetch permissions, returning None if an error occurred
    let (user_perms, bot_perms) = user_permissions(ctx, user_id, bot_id).await?;

    let user_missing_perms = user_perms
        .map(|permissions| user.1 - permissions)
        .unwrap_or_default();
    let bot_missing_perms = bot_perms
        .map(|permissions| user.1 - permissions)
        .unwrap_or_default();

    Some((user_missing_perms, bot_missing_perms))
}

/// See [`check_permissions_and_cooldown`]. Runs the check only for a single command. The caller
/// should call this multiple time for each parent command to achieve the check inheritance logic.
async fn check_permissions_and_cooldown_single<'a, U, E>(
    ctx: crate::Context<'a, U, E>,
    cmd: &'a crate::Command<U, E>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    // Skip command checks if `FrameworkOptions::skip_checks_for_owners` is set to true
    if ctx.framework().options.skip_checks_for_owners
        && ctx.framework().options().owners.contains(&ctx.author().id)
    {
        return Ok(());
    }

    if cmd.owners_only && !ctx.framework().options().owners.contains(&ctx.author().id) {
        return Err(crate::FrameworkError::NotAnOwner { ctx });
    }

    if cmd.guild_only {
        match ctx.guild_id() {
            None => return Err(crate::FrameworkError::GuildOnly { ctx }),
            Some(guild_id) => {
                #[cfg(feature = "cache")]
                if ctx.framework().options().require_cache_for_guild_check
                    && ctx.cache().guild(guild_id).is_none()
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
        let channel = match ctx.channel_id().to_channel(ctx.serenity_context()).await {
            Ok(channel) => channel,
            Err(e) => {
                tracing::warn!("Error when getting channel: {}", e);

                return Err(crate::FrameworkError::NsfwOnly { ctx });
            }
        };

        if let serenity::Channel::Guild(guild_channel) = channel {
            if !guild_channel.nsfw {
                return Err(crate::FrameworkError::NsfwOnly { ctx });
            }
        }
    }

    // Make sure that user has required permissions
    if let Some((user_missing_permissions, bot_missing_permissions)) = missing_permissions(
        ctx,
        (ctx.author().id, cmd.required_permissions),
        (ctx.framework().bot_id, cmd.required_bot_permissions),
    )
    .await
    {
        if !user_missing_permissions.is_empty() {
            return Err(crate::FrameworkError::MissingUserPermissions {
                ctx,
                missing_permissions: Some(user_missing_permissions),
            });
        }

        if !bot_missing_permissions.is_empty() {
            return Err(crate::FrameworkError::MissingBotPermissions {
                ctx,
                missing_permissions: bot_missing_permissions,
            });
        }

        // missing premission checks here.
    } else {
        // TODO: ask what I should do here because combining the checks loses the verbosity.
        // the only previous failure point was it failing to get the guild, channel or members.
        // Previously when a bots permissions could not be fetched it would just allow execution.
        return Err(crate::FrameworkError::MissingUserPermissions {
            missing_permissions: None,
            ctx,
        });
    }

    // Only continue if command checks returns true
    // First perform global checks, then command checks (if necessary)
    for check in Option::iter(&ctx.framework().options().command_check).chain(&cmd.checks) {
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
        let cooldowns = cmd.cooldowns.lock().unwrap();
        let config = cmd.cooldown_config.read().unwrap();
        let remaining_cooldown = cooldowns.remaining_cooldown(ctx.cooldown_context(), &config);
        if let Some(remaining_cooldown) = remaining_cooldown {
            return Err(crate::FrameworkError::CooldownHit {
                ctx,
                remaining_cooldown,
            });
        }
    }

    Ok(())
}

/// Checks if the invoker is allowed to execute this command at this point in time
///
/// Doesn't actually start the cooldown timer! This should be done by the caller later, after
/// argument parsing.
/// (A command that didn't even get past argument parsing shouldn't trigger cooldowns)
#[allow(clippy::needless_lifetimes)] // false positive (clippy issue 7271)
pub async fn check_permissions_and_cooldown<'a, U, E>(
    ctx: crate::Context<'a, U, E>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    for parent_command in ctx.parent_commands() {
        check_permissions_and_cooldown_single(ctx, parent_command).await?;
    }
    check_permissions_and_cooldown_single(ctx, ctx.command()).await?;

    Ok(())
}
