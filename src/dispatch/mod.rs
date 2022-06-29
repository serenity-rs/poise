//! Contains all code to dispatch incoming events onto framework commands

mod common;
mod prefix;
mod slash;

pub use prefix::{dispatch_message, find_command};

use crate::serenity_prelude as serenity;

// TODO: integrate serenity::Context in here? Every place where FrameworkContext is passed is also
// passed serenity::Context
/// A view into data stored by [`crate::Framework`]
pub struct FrameworkContext<'a, U, E> {
    /// User ID of this bot
    pub bot_id: serenity::UserId,
    /// Framework configuration
    pub options: &'a crate::FrameworkOptions<U, E>,
    /// Your provided user data
    pub user_data: &'a U,
    /// Serenity shard manager. Can be used for example to shutdown the bot
    pub shard_manager: &'a std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>>,
    // deliberately not non exhaustive because you need to create FrameworkContext from scratch
    // to run your own event loop
}
impl<U, E> Copy for FrameworkContext<'_, U, E> {}
impl<U, E> Clone for FrameworkContext<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, U, E> FrameworkContext<'a, U, E> {
    /// Returns the stored framework options, including commands.
    pub fn options(&self) -> &'a crate::FrameworkOptions<U, E> {
        self.options
    }

    /// Returns the serenity's client shard manager.
    pub fn shard_manager(&self) -> std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>> {
        self.shard_manager.clone()
    }

    /// Retrieves user data
    pub async fn user_data(&self) -> &'a U {
        self.user_data
    }
}

/// Central event handling function of this library
pub async fn dispatch_event<U: Send + Sync, E>(
    framework: crate::FrameworkContext<'_, U, E>,
    ctx: &serenity::Context,
    event: &crate::Event<'_>,
) {
    match event {
        crate::Event::Message { new_message } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            if let Err(Some((error, command))) = prefix::dispatch_message(
                framework,
                ctx,
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
                        ctx,
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
                ctx,
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
                ctx,
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
        (framework.options.listener)(ctx, event, framework, framework.user_data().await).await
    {
        let error = crate::FrameworkError::Listener {
            ctx: ctx.clone(),
            error,
            event,
            framework,
        };
        (framework.options.on_error)(error).await;
    }
}
