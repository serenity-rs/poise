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
    event: &crate::Event<'_>,
) where
    U: Send + Sync,
{
    match event {
        crate::Event::Ready { data_about_bot } => {
            let user_data_setup = Option::take(&mut *framework.user_data_setup.lock().unwrap());
            if let Some(user_data_setup) = user_data_setup {
                match user_data_setup(&ctx, data_about_bot, framework).await {
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
        crate::Event::Message { new_message } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            if let Err(Some((error, command))) = prefix::dispatch_message(
                framework,
                &ctx,
                new_message,
                false,
                false,
                &invocation_data,
            )
            .await
            {
                command.on_error.unwrap_or(framework.options.on_error)(error).await;
            }
        }
        crate::Event::MessageUpdate { event, .. } => {
            if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
                let msg = edit_tracker.write().unwrap().process_message_update(
                    event,
                    framework
                        .options()
                        .prefix_options
                        .ignore_edits_if_not_yet_responded,
                );

                if let Some((msg, previously_tracked)) = msg {
                    let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
                    if let Err(Some((error, command))) = prefix::dispatch_message(
                        framework,
                        &ctx,
                        &msg,
                        true,
                        previously_tracked,
                        &invocation_data,
                    )
                    .await
                    {
                        command.on_error.unwrap_or(framework.options.on_error)(error).await;
                    }
                }
            }
        }
        crate::Event::InteractionCreate {
            interaction: serenity::Interaction::ApplicationCommand(interaction),
        } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            if let Err(Some((error, command))) = slash::dispatch_interaction(
                framework,
                &ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
                &invocation_data,
            )
            .await
            {
                command.on_error.unwrap_or(framework.options.on_error)(error).await;
            }
        }
        crate::Event::InteractionCreate {
            interaction: serenity::Interaction::Autocomplete(interaction),
        } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            if let Err(Some((error, command))) = slash::dispatch_autocomplete(
                framework,
                &ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
                &invocation_data,
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
        let error = crate::FrameworkError::Listener {
            ctx,
            error,
            event,
            framework,
        };
        (framework.options.on_error)(error).await;
    }
}
