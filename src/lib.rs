#![warn(rust_2018_idioms)]
#![allow(clippy::type_complexity)]

mod event;
pub use event::{Event, EventWrapper};

mod argument;
pub use argument::*;

mod error;
pub use error::*;

mod track_edits;
pub use track_edits::*;

pub mod utils;

pub use poise_macros::*;

mod serenity {
    pub use serenity::{
        builder::*,
        client::bridge::gateway::*,
        model::{event::*, prelude::*},
        prelude::*,
        utils::*,
        Error,
    };
}

use std::future::Future;
use std::pin::Pin;

/// Shorthand for a wrapped async future with a lifetime, used by many parts of this framework.
///
/// An owned future has the `'static` lifetime.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// needed for proc macro
#[doc(hidden)]
pub trait _GetGenerics {
    type U;
    type E;
}

/// Passed to command invocations.
///
/// Contains the trigger message, the Discord connection management stuff, and the user data.
pub struct Context<'a, U, E> {
    pub discord: &'a serenity::Context,
    pub msg: &'a serenity::Message,
    pub framework: &'a Framework<U, E>,
    pub data: &'a U,
}
// manual Copy+Clone implementations because Rust is getting confused about the type parameter
impl<U, E> Clone for Context<'_, U, E> {
    fn clone(&self) -> Self {
        Self {
            discord: self.discord,
            msg: self.msg,
            framework: self.framework,
            data: self.data,
        }
    }
}
impl<U, E> Copy for Context<'_, U, E> {}
impl<U_, E_> _GetGenerics for Context<'_, U_, E_> {
    type U = U_;
    type E = E_;
}

pub struct CommandOptions<U, E> {
    /// Short description of the command. Displayed inline in help menus and similar.
    pub description: Option<&'static str>,
    /// Multiline description with detailed usage instructions. Displayed in the command specific
    /// help: `~help command_name`
    // TODO: fix the inconsistency that this is String and everywhere else it's &'static str
    pub explanation: Option<fn() -> String>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Fall back to the framework-specified value on None.
    pub on_error: Option<fn(E, CommandErrorContext<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// Alternative triggers for the command
    pub aliases: &'static [&'static str],
    /// Whether to enable edit tracking for commands by default. Note that this won't do anything
    /// if `Framework::edit_tracker` isn't set.
    pub track_edits: bool,
    /// Fall back to the framework-specified value on None.
    pub broadcast_typing: Option<bool>,
}

impl<U, E> Default for CommandOptions<U, E> {
    fn default() -> Self {
        Self {
            description: None,
            explanation: None,
            check: None,
            on_error: None,
            aliases: &[],
            track_edits: false,
            broadcast_typing: None,
        }
    }
}

pub struct Command<U, E> {
    pub name: &'static str,
    pub action: for<'a> fn(Context<'a, U, E>, args: &'a str) -> BoxFuture<'a, Result<(), E>>,
    pub options: CommandOptions<U, E>,
}

pub struct FrameworkOptions<U, E> {
    /// List of bot commands.
    pub commands: Vec<Command<U, E>>,
    /// List of additional bot prefixes
    pub additional_prefixes: &'static [&'static str],
    /// Provide a callback to be invoked when any user code yields an error.
    // pub on_error: fn(E, ErrorContext<'_, U, E>) -> BoxFuture<()>,
    pub on_error: fn(E, ErrorContext<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// Individual commands may override this callback.
    pub command_check: fn(Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>,
    /// Called on every Discord event. Can be used to react to non-command events, like messages
    /// deletions or guild updates.
    pub listener: for<'a> fn(
        &'a serenity::Context,
        &'a Event<'a>,
        &'a Framework<U, E>,
        &'a U,
    ) -> BoxFuture<'a, Result<(), E>>,
    /// If Some, the framework will react to message edits by editing the corresponding bot response
    /// with the new result.
    pub edit_tracker: Option<parking_lot::RwLock<EditTracker>>,
    /// Whether to broadcast a typing indicator while executing this commmand's action.
    pub broadcast_typing: bool,
}

impl<U, E> Default for FrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            additional_prefixes: &[],
            on_error: |_, _| {
                Box::pin(async {
                    println!("Discord bot framework encountered an error in user code");
                })
            },
            command_check: |_| Box::pin(async { Ok(true) }),
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            edit_tracker: None,
            broadcast_typing: false,
        }
    }
}

pub struct Framework<U, E> {
    prefix: &'static str,
    user_data: once_cell::sync::OnceCell<U>,
    user_data_setup: std::sync::Mutex<
        Option<Box<dyn Send + Sync + FnOnce(&serenity::Context, &serenity::Ready) -> U>>,
    >,
    // TODO: wrap in RwLock to allow changing framework options while running? Could also replace
    // the edit tracking cache interior mutability
    options: FrameworkOptions<U, E>,
}

impl<U, E> Framework<U, E>
where
    U: Send + Sync + 'static,
    E: 'static + Send,
{
    /// Setup a new blank Framework with a prefix and a callback to provide user data.
    ///
    /// The user data callback is invoked as soon as the bot is logged. That way, bot data like user
    /// ID or connected guilds can be made available to the user data setup function. The user data
    /// setup is not allowed to return Result because there would be no reasonable
    /// course of action on error.
    pub fn new<F: FnOnce(&serenity::Context, &serenity::Ready) -> U>(
        prefix: &'static str,
        user_data_setup: F,
        options: FrameworkOptions<U, E>,
    ) -> Self
    where
        F: Send + Sync + 'static + FnOnce(&serenity::Context, &serenity::Ready) -> U,
    {
        Self {
            prefix,
            user_data: once_cell::sync::OnceCell::new(),
            user_data_setup: std::sync::Mutex::new(Some(Box::new(user_data_setup))),
            options,
        }
    }

    pub async fn start(self, token: &str) -> Result<(), serenity::Error> {
        let self_1 = std::sync::Arc::new(self);
        let self_2 = std::sync::Arc::clone(&self_1);

        let edit_track_cache_purge_task = tokio::spawn(async move {
            loop {
                if let Some(edit_tracker) = &self_1.options.edit_tracker {
                    edit_tracker.write().purge();
                }
                // not sure if the purging interval should be configurable
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        let event_handler = EventWrapper(move |ctx, event| {
            let self_2 = std::sync::Arc::clone(&self_2);
            Box::pin(async move {
                self_2.event(ctx, event).await;
            }) as _
        });
        serenity::Client::builder(token)
            .intents(
                serenity::GatewayIntents::non_privileged()
                    | serenity::GatewayIntents::GUILD_MEMBERS
                    | serenity::GatewayIntents::GUILD_PRESENCES,
            )
            .event_handler(event_handler)
            .await?
            .start()
            .await?;

        edit_track_cache_purge_task.abort();

        Ok(())
    }

    pub fn options(&self) -> &FrameworkOptions<U, E> {
        &self.options
    }

    fn get_user_data(&self) -> &U {
        // We shouldn't get a Message event before a Ready event. But if we do, wait until
        // the Ready event does come and the resulting data has arrived.
        loop {
            match self.user_data.get() {
                Some(x) => break x,
                None => std::thread::sleep(std::time::Duration::from_millis(100)),
            }
        }
    }

    /// Returns
    /// - Ok(()) if a command was successfully dispatched
    /// - Err(None) if the message does not match any known command
    /// - Err(Some(error: UserError)) if any user code yielded an error
    async fn dispatch_message<'a>(
        &'a self,
        ctx: Context<'a, U, E>,
        triggered_by_edit: bool,
    ) -> Result<(), Option<(E, ErrorContext<'a, U, E>)>> {
        // Check prefix
        let msg = match ctx.msg.content.strip_prefix(self.prefix) {
            Some(msg) => msg,
            None => self
                .options
                .additional_prefixes
                .iter()
                .find_map(|prefix| ctx.msg.content.strip_prefix(prefix))
                .ok_or(None)?,
        };

        // Extract command name and arguments string
        let (command_name, args) = {
            let mut iter = msg.splitn(2, ' ');
            (iter.next().unwrap(), iter.next().unwrap_or(""))
        };

        // Find the first matching command
        let mut first_matching_command = None;
        for command in &self.options.commands {
            if command.name != command_name {
                continue;
            }
            match (command.options.check.unwrap_or(self.options.command_check))(ctx).await {
                Ok(true) => {}
                Ok(false) => continue,
                Err(e) => {
                    return Err(Some((
                        e,
                        ErrorContext::Command(CommandErrorContext {
                            command,
                            ctx,
                            while_checking: true,
                        }),
                    )));
                }
            }

            first_matching_command = Some(command);
            break;
        }
        let command = first_matching_command.ok_or(None)?;

        if triggered_by_edit && !command.options.track_edits {
            return Err(None);
        }

        if command
            .options
            .broadcast_typing
            .unwrap_or(self.options.broadcast_typing)
        {
            let _: Result<_, _> = ctx.msg.channel_id.broadcast_typing(ctx.discord).await;
        }

        // Execute command
        (command.action)(ctx, args).await.map_err(|e| {
            Some((
                e,
                ErrorContext::Command(CommandErrorContext {
                    ctx,
                    command,
                    while_checking: false,
                }),
            ))
        })
    }

    async fn event(&self, ctx: serenity::Context, event: Event<'_>) {
        match &event {
            Event::Ready { data_about_bot } => match self.user_data_setup.lock().unwrap().take() {
                Some(user_data_setup) => {
                    let _: Result<_, _> =
                        self.user_data.set(user_data_setup(&ctx, &data_about_bot));
                }
                None => println!("Warning: skipping duplicate Discord bot ready event"),
            },
            Event::Message { new_message } => {
                let ctx = Context {
                    discord: &ctx,
                    msg: &new_message,
                    framework: self,
                    data: self.get_user_data(),
                };
                if let Err(Some((err, err_ctx))) = self.dispatch_message(ctx, false).await {
                    match err_ctx.clone() {
                        ErrorContext::Command(command_err_ctx) => {
                            if let Some(on_error) = command_err_ctx.command.options.on_error {
                                (on_error)(err, command_err_ctx).await;
                            } else {
                                (self.options.on_error)(err, err_ctx).await;
                            }
                        }
                        err_ctx => (self.options.on_error)(err, err_ctx).await,
                    }
                }
            }
            Event::MessageUpdate { event, .. } => {
                if let Some(edit_tracker) = &self.options.edit_tracker {
                    let msg = edit_tracker.write().process_message_update(event);

                    let ctx = Context {
                        discord: &ctx,
                        msg: &msg,
                        framework: self,
                        data: self.get_user_data(),
                    };
                    if let Err(Some((err, err_ctx))) = self.dispatch_message(ctx, true).await {
                        (self.options.on_error)(err, err_ctx);
                    }
                }
            }
            Event::MessageDelete {
                deleted_message_id, ..
            } => {
                if let Some(edit_tracker) = &self.options.edit_tracker {
                    let bot_response = edit_tracker
                        .write()
                        .find_bot_response(*deleted_message_id)
                        .cloned();
                    if let Some(bot_response) = bot_response {
                        if let Err(e) = bot_response.delete(&ctx).await {
                            println!(
                                "Warning: couldn't delete bot response when user deleted message: {}",
                                e
                            );
                        }
                    }
                }
            }
            _ => {}
        }

        // Do this after the framework's Ready handling, so that self.get_user_data() doesnt
        // potentially block infinitely
        if let Err(e) = (self.options.listener)(&ctx, &event, self, self.get_user_data()).await {
            (self.options.on_error)(e, ErrorContext::Listener(&event));
        }
    }
}
