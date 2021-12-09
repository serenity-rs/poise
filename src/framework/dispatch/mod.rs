// Prefix and slash specific implementation details
mod common;
mod prefix;
mod slash;

pub use prefix::dispatch_message;

use crate::serenity_prelude as serenity;

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
                if let Some(on_error) = ctx.command.id.on_error {
                    (on_error)(err, crate::CommandErrorContext::Prefix(ctx)).await;
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
                        let ctx = crate::CommandErrorContext::Prefix(ctx);
                        if let Some(on_error) = ctx.command().id().on_error {
                            on_error(err, ctx).await;
                        } else {
                            (framework.options.on_error)(err, crate::ErrorContext::Command(ctx))
                                .await;
                        }
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
                if let Some(on_error) = error_ctx.ctx.command.id().on_error {
                    on_error(e, crate::CommandErrorContext::Application(error_ctx)).await;
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
                if let Some(on_error) = error_ctx.ctx.command.id().on_error {
                    on_error(e, crate::CommandErrorContext::Application(error_ctx)).await;
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
