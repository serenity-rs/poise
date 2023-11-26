//! Contains all code to dispatch incoming events onto framework commands

mod common;
mod prefix;
mod slash;

pub use common::*;
pub use prefix::*;
pub use slash::*;

use crate::serenity_prelude as serenity;

// TODO: integrate serenity::Context in here? Every place where FrameworkContext is passed is also
// passed serenity::Context
/// A view into data stored by [`crate::Framework`]
pub struct FrameworkContext<'a, U: Send + Sync + 'static, E> {
    /// User ID of this bot
    pub bot_id: serenity::UserId,
    /// Framework configuration
    pub options: &'a crate::FrameworkOptions<U, E>,
    /// Serenity shard manager. Can be used for example to shutdown the bot
    pub shard_manager: &'a std::sync::Arc<serenity::ShardManager>,
    // deliberately not non exhaustive because you need to create FrameworkContext from scratch
    // to run your own event loop
}
impl<U: Send + Sync + 'static, E> Copy for FrameworkContext<'_, U, E> {}
impl<U: Send + Sync + 'static, E> Clone for FrameworkContext<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, U: Send + Sync + 'static, E> FrameworkContext<'a, U, E> {
    /// Returns the stored framework options, including commands.
    ///
    /// This function exists for API compatiblity with [`crate::Framework`]. On this type, you can
    /// also just access the public `options` field.
    pub fn options(&self) -> &'a crate::FrameworkOptions<U, E> {
        self.options
    }

    /// Returns the serenity's client shard manager.
    ///
    /// This function exists for API compatiblity with [`crate::Framework`]. On this type, you can
    /// also just access the public `shard_manager` field.
    pub fn shard_manager(&self) -> std::sync::Arc<serenity::ShardManager> {
        self.shard_manager.clone()
    }
}

/// Central event handling function of this library
pub async fn dispatch_event<U: Send + Sync, E>(
    framework: crate::FrameworkContext<'_, U, E>,
    ctx: &serenity::Context<U>,
    event: serenity::FullEvent,
) {
    match &event {
        serenity::FullEvent::Message { new_message } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            let mut parent_commands = Vec::new();
            let trigger = crate::MessageDispatchTrigger::MessageCreate;
            if let Err(error) = prefix::dispatch_message(
                framework,
                ctx,
                new_message,
                trigger,
                &invocation_data,
                &mut parent_commands,
            )
            .await
            {
                error.handle(framework.options).await;
            }
        }
        serenity::FullEvent::MessageUpdate { event, .. } => {
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
                    let mut parent_commands = Vec::new();
                    let trigger = match previously_tracked {
                        true => crate::MessageDispatchTrigger::MessageEdit,
                        false => crate::MessageDispatchTrigger::MessageEditFromInvalid,
                    };
                    if let Err(error) = prefix::dispatch_message(
                        framework,
                        ctx,
                        &msg,
                        trigger,
                        &invocation_data,
                        &mut parent_commands,
                    )
                    .await
                    {
                        error.handle(framework.options).await;
                    }
                }
            }
        }
        serenity::FullEvent::MessageDelete {
            deleted_message_id, ..
        } => {
            if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
                let bot_response = edit_tracker
                    .write()
                    .unwrap()
                    .process_message_delete(*deleted_message_id);
                if let Some(bot_response) = bot_response {
                    if let Err(e) = bot_response.delete(&ctx).await {
                        tracing::warn!("failed to delete bot response: {}", e);
                    }
                }
            }
        }
        serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Command(interaction),
        } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            let mut parent_commands = Vec::new();
            if let Err(error) = slash::dispatch_interaction(
                framework,
                ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
                &invocation_data,
                &interaction.data.options(),
                &mut parent_commands,
            )
            .await
            {
                error.handle(framework.options).await;
            }
        }
        serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Autocomplete(interaction),
        } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            let mut parent_commands = Vec::new();
            if let Err(error) = slash::dispatch_autocomplete(
                framework,
                ctx,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
                &invocation_data,
                &interaction.data.options(),
                &mut parent_commands,
            )
            .await
            {
                error.handle(framework.options).await;
            }
        }
        _ => {}
    }

    if let Err(error) = (framework.options.event_handler)(ctx, &event, framework).await {
        let error = crate::FrameworkError::EventHandler {
            ctx,
            error,
            event: &event,
            framework,
        };
        (framework.options.on_error)(error).await;
    }
}
