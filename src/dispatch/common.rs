//! Prefix and slash agnostic utilities for dispatching incoming events onto framework commands

#[cfg(feature = "cache")]
use crate::serenity_prelude as serenity;

/// Fetches the NSFW status of the channel (or thread) the command was executed in.
#[cfg(feature = "cache")]
async fn check_nsfw_channel<U: Send + Sync + 'static, E>(ctx: crate::Context<'_, U, E>) -> bool {
    match ctx.channel().await {
        Some(serenity::Channel::Private(_)) => true,
        Some(serenity::Channel::Guild(ch)) => ch.nsfw,
        Some(serenity::Channel::GuildThread(th)) => {
            let guild_id = th.base.guild_id;
            match th.parent_id.to_guild_channel(ctx, Some(guild_id)).await {
                Ok(ch) => ch.nsfw,
                Err(e) => {
                    tracing::warn!("Error when getting thread parent for NSFW check: {e}");
                    false
                },
            }
        },
        None | Some(_) => {
            tracing::warn!("Error when getting channel for NSFW check");
            false
        },
    }
}

/// See [`check_permissions_and_cooldown`]. Runs the check only for a single command. The caller
/// should call this multiple time for each parent command to achieve the check inheritance logic.
async fn check_permissions_and_cooldown_single<'a, U: Send + Sync + 'static, E>(
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
            },
        }
    }

    if cmd.dm_only && ctx.guild_id().is_some() {
        return Err(crate::FrameworkError::DmOnly { ctx });
    }

    #[cfg(feature = "cache")]
    {
        if cmd.nsfw_only && !check_nsfw_channel(ctx).await {
            return Err(crate::FrameworkError::NsfwOnly { ctx });
        }
    }

    // Make sure that user has required permissions
    if let Some((user_missing_permissions, bot_missing_permissions)) =
        super::permissions::calculate_missing(
            ctx,
            cmd.required_permissions,
            cmd.required_bot_permissions,
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
    } else {
        return Err(crate::FrameworkError::PermissionFetchFailed { ctx });
    }

    // Only continue if command checks returns true
    // First perform global checks, then command checks (if necessary)
    for check in Option::iter(&ctx.framework().options().command_check).chain(&cmd.checks) {
        match check(ctx).await {
            Ok(true) => {},
            Ok(false) => {
                return Err(crate::FrameworkError::CommandCheckFailed { ctx, error: None });
            },
            Err(error) => {
                return Err(crate::FrameworkError::CommandCheckFailed { error: Some(error), ctx });
            },
        }
    }

    if !ctx.framework().options().manual_cooldowns {
        let cooldowns = cmd.cooldowns.lock().unwrap();
        let config = cmd.cooldown_config.read().unwrap();
        let remaining_cooldown = cooldowns.remaining_cooldown(ctx.cooldown_context(), &config);
        if let Some(remaining_cooldown) = remaining_cooldown {
            return Err(crate::FrameworkError::CooldownHit { ctx, remaining_cooldown });
        }
    }

    Ok(())
}

/// Checks if the invoker is allowed to execute this command at this point in time
///
/// Doesn't actually start the cooldown timer! This should be done by the caller later, after
/// argument parsing.
/// (A command that didn't even get past argument parsing shouldn't trigger cooldowns)
pub async fn check_permissions_and_cooldown<U: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, U, E>,
) -> Result<(), crate::FrameworkError<'_, U, E>> {
    for command in ctx.command_tree() {
        check_permissions_and_cooldown_single(ctx, command).await?;
    }

    Ok(())
}
