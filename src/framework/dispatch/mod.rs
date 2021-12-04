// Prefix and slash specific implementation details
mod prefix;
mod slash;

pub use prefix::dispatch_message;

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

pub async fn dispatch_event<U, E>(
    framework: &crate::Framework<U, E>,
    ctx: serenity::Context,
    event: crate::Event<'_>,
) where
    U: Send + Sync,
{
    match &event {
        crate::Event::Ready { data_about_bot } => {
            let user_data_setup = Option::take(&mut *framework.user_data_setup.lock().unwrap());
            if let Some(user_data_setup) = user_data_setup {
                match user_data_setup(&ctx, data_about_bot, framework).await {
                    Ok(user_data) => {
                        let _: Result<_, _> = framework.user_data.set(user_data);
                    }
                    Err(e) => (framework.options.on_error)(e, crate::ErrorContext::Setup).await,
                }
            } else {
                // discarding duplicate Discord bot ready event
                // (happens regularly when bot is online for long period of time)
            }
        }
        crate::Event::Message { new_message } => {
            if let Err(Some((err, ctx))) =
                prefix::dispatch_message(framework, &ctx, new_message, false, false).await
            {
                if let Some(on_error) = ctx.command.options.on_error {
                    (on_error)(err, ctx).await;
                } else {
                    (framework.options.on_error)(
                        err,
                        crate::ErrorContext::Command(crate::CommandErrorContext::Prefix(ctx)),
                    )
                    .await;
                }
            }
        }
        crate::Event::MessageUpdate { event, .. } => {
            if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
                let msg = edit_tracker.write().unwrap().process_message_update(
                    event,
                    framework.options().prefix_options.ignore_edit_tracker_cache,
                );

                if let Some((msg, previously_tracked)) = msg {
                    if let Err(Some((err, ctx))) =
                        prefix::dispatch_message(framework, &ctx, &msg, true, previously_tracked)
                            .await
                    {
                        (framework.options.on_error)(
                            err,
                            crate::ErrorContext::Command(crate::CommandErrorContext::Prefix(ctx)),
                        )
                        .await;
                    }
                }
            }
        }
        crate::Event::InteractionCreate {
            interaction: serenity::Interaction::ApplicationCommand(interaction),
        } => {
            if let Err(Some((e, error_ctx))) = slash::dispatch_interaction(
                framework,
                &ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
            )
            .await
            {
                if let Some(on_error) = error_ctx.ctx.command.options().on_error {
                    on_error(e, error_ctx).await;
                } else {
                    (framework.options.on_error)(
                        e,
                        crate::ErrorContext::Command(crate::CommandErrorContext::Application(
                            error_ctx,
                        )),
                    )
                    .await;
                }
            }
        }
        crate::Event::InteractionCreate {
            interaction: serenity::Interaction::Autocomplete(interaction),
        } => {
            if let Err(Some((e, error_ctx))) = slash::dispatch_autocomplete(
                framework,
                &ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
            )
            .await
            {
                if let Some(on_error) = error_ctx.ctx.command.options().on_error {
                    on_error(e, error_ctx).await;
                } else {
                    (framework.options.on_error)(e, crate::ErrorContext::Autocomplete(error_ctx))
                        .await;
                }
            }
        }
        _ => {}
    }

    // Do this after the framework's Ready handling, so that get_user_data() doesnt
    // potentially block infinitely
    if let Err(e) =
        (framework.options.listener)(&ctx, &event, framework, framework.get_user_data().await).await
    {
        (framework.options.on_error)(e, crate::ErrorContext::Listener(&event)).await;
    }
}
