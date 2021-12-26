//! Contains all code to dispatch incoming events onto framework commands

mod common;
mod prefix;
mod slash;

pub use prefix::{dispatch_message, find_command};

use crate::serenity_prelude as serenity;

/// Central event handling function of this library
pub async fn dispatch_event<U, E>(
    framework: &crate::Framework<U, E>,
    ctx: serenity::Context,
    event: &serenity::Event,
) where
    U: Send + Sync,
{
    match &event {
        serenity::Event::Ready(serenity::ReadyEvent { ready, .. }) => {
            let user_data_setup = Option::take(&mut *framework.user_data_setup.lock().unwrap());
            if let Some(user_data_setup) = user_data_setup {
                match user_data_setup(&ctx, ready, framework).await {
                    Ok(user_data) => {
                        let _: Result<_, _> = framework.user_data.set(user_data);
                    }
                    Err(error) => {
                        (framework.options.on_error)(crate::FrameworkError::Setup { error }).await
                    }
                }
            } else {
                // discarding duplicate Discord bot ready event
                // (happens regularly when bot is online for long period of time)
            }
        }
        serenity::Event::MessageCreate(serenity::MessageCreateEvent { message, .. }) => {
            if let Err(Some((error, command))) =
                prefix::dispatch_message(framework, &ctx, message, false, false).await
            {
                command.on_error.unwrap_or(framework.options.on_error)(error).await;
            }
        }
        serenity::Event::MessageUpdate(event) => {
            if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
                let msg = edit_tracker.write().unwrap().process_message_update(
                    event,
                    framework.options().prefix_options.ignore_edit_tracker_cache,
                );

                if let Some((msg, previously_tracked)) = msg {
                    if let Err(Some((error, command))) =
                        prefix::dispatch_message(framework, &ctx, &msg, true, previously_tracked)
                            .await
                    {
                        command.on_error.unwrap_or(framework.options.on_error)(error).await;
                    }
                }
            }
        }
        serenity::Event::InteractionCreate(serenity::InteractionCreateEvent {
            interaction: serenity::Interaction::ApplicationCommand(interaction),
            ..
        }) => {
            if let Err(Some((error, command))) = slash::dispatch_interaction(
                framework,
                &ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
            )
            .await
            {
                command.on_error.unwrap_or(framework.options.on_error)(error).await;
            }
        }
        serenity::Event::InteractionCreate(serenity::InteractionCreateEvent {
            interaction: serenity::Interaction::Autocomplete(interaction),
            ..
        }) => {
            if let Err(Some((error, command))) = slash::dispatch_autocomplete(
                framework,
                &ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
            )
            .await
            {
                command.on_error.unwrap_or(framework.options.on_error)(error).await;
            }
        }
        _ => {}
    }

    // Do this after the framework's Ready handling, so that get_user_data() doesnt
    // potentially block infinitely
    if let Err(error) =
        (framework.options.listener)(&ctx, event, framework, framework.user_data().await).await
    {
        let error = crate::FrameworkError::Listener { error, event };
        (framework.options.on_error)(error).await;
    }
}
