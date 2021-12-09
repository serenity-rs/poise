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

    let channel = match guild.channels.get(&channel_id) {
        Some(serenity::Channel::Guild(channel)) => channel,
        Some(_other_channel) => {
            println!(
                "Warning: guild message was supposedly sent in a non-guild channel. Denying invocation"
            );
            return None;
        }
        None => return None,
    };

    // If member not in cache (probably because presences intent is not enabled), retrieve via HTTP
    let member = match guild.members.get(&user_id) {
        Some(x) => x.clone(),
        None => match ctx.http.get_member(guild_id.0, user_id.0).await {
            Ok(member) => member,
            Err(_) => return None,
        },
    };

    guild.user_permissions_in(channel, &member).ok()
}

async fn check_required_permissions_and_owners_only<U, E>(
    ctx: crate::Context<'_, U, E>,
    required_permissions: serenity::Permissions,
    owners_only: bool,
) -> bool {
    if owners_only && !ctx.framework().options().owners.contains(&ctx.author().id) {
        return false;
    }

    if !required_permissions.is_empty() {
        let user_permissions = user_permissions(
            ctx.discord(),
            ctx.guild_id(),
            ctx.channel_id(),
            ctx.discord().cache.current_user_id(),
        )
        .await;
        match user_permissions {
            Some(perms) => {
                if !perms.contains(required_permissions) {
                    return false;
                }
            }
            // better safe than sorry: when perms are unknown, restrict access
            None => return false,
        }
    }

    true
}

async fn check_missing_bot_permissions<U, E>(
    ctx: crate::Context<'_, U, E>,
    required_bot_permissions: serenity::Permissions,
) -> serenity::Permissions {
    let user_permissions = user_permissions(
        ctx.discord(),
        ctx.guild_id(),
        ctx.channel_id(),
        ctx.discord().cache.current_user_id(),
    )
    .await;
    match user_permissions {
        Some(perms) => required_bot_permissions - perms,
        // When in doubt, just let it run. Not getting fancy missing permissions errors is better
        // than the command not executing at all
        None => serenity::Permissions::empty(),
    }
}

pub async fn check_permissions_and_cooldown<U, E>(
    ctx: crate::Context<'_, U, E>,
    cmd: &crate::CommandId<U, E>,
) -> Result<bool, (E, crate::CommandErrorLocation)> {
    // Make sure that user has required permissions
    if !check_required_permissions_and_owners_only(ctx, cmd.required_permissions, cmd.owners_only)
        .await
    {
        return Ok(false);
    }

    // Before running any pre-command checks, make sure the bot has the permissions it needs
    let missing_bot_permissions =
        check_missing_bot_permissions(ctx, cmd.required_bot_permissions).await;
    if !missing_bot_permissions.is_empty() {
        (ctx.framework().options().missing_bot_permissions_handler)(ctx, missing_bot_permissions)
            .await
            .map_err(|e| {
                (
                    e,
                    crate::CommandErrorLocation::MissingBotPermissionsCallback,
                )
            })?;
        return Ok(false);
    }

    // Only continue if command checks returns true
    let checks_passing = (|| async {
        let global_check_passes = match &ctx.framework().options().command_check {
            Some(check) => check(ctx).await?,
            None => true,
        };

        let command_specific_check_passes = match &cmd.check {
            Some(check) => check(ctx).await?,
            None => true,
        };

        Ok(global_check_passes && command_specific_check_passes)
    })()
    .await
    .map_err(|e| (e, crate::CommandErrorLocation::Check))?;
    if !checks_passing {
        return Ok(false);
    }

    let cooldowns = &cmd.cooldowns;
    let cooldown_left = cooldowns.lock().unwrap().get_wait_time(ctx);
    if let Some(cooldown_left) = cooldown_left {
        if let Some(callback) = ctx.framework().options().cooldown_hit {
            callback(ctx, cooldown_left)
                .await
                .map_err(|e| (e, crate::CommandErrorLocation::CooldownCallback))?;
        }
        return Ok(false);
    }
    cooldowns.lock().unwrap().start_cooldown(ctx);

    Ok(true)
}
