//! Prefix and slash agnostic utilities for dispatching incoming events onto framework commands

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
            }
        }
    }

    if cmd.dm_only && ctx.guild_id().is_some() {
        return Err(crate::FrameworkError::DmOnly { ctx });
    }

    if cmd.nsfw_only {
        if let Some(guild_id) = ctx.guild_id() {
            let serenity_ctx = ctx.serenity_context();
            let channel_id = ctx.channel_id();
            let channel = match channel_id
                .to_guild_channel(serenity_ctx, Some(guild_id))
                .await
            {
                Ok(channel) => channel,
                Err(e) => {
                    tracing::warn!("Error when getting channel: {}", e);
                    return Err(crate::FrameworkError::NsfwOnly { ctx });
                }
            };

            if !channel.nsfw {
                return Err(crate::FrameworkError::NsfwOnly { ctx });
            }
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
pub async fn check_permissions_and_cooldown<'a, U: Send + Sync + 'static, E>(
    ctx: crate::Context<'a, U, E>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    for parent_command in ctx.parent_commands() {
        check_permissions_and_cooldown_single(ctx, parent_command).await?;
    }
    check_permissions_and_cooldown_single(ctx, ctx.command()).await?;

    Ok(())
}
