//! Holds application command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture, Framework};

/// Application command specific context passed to command invocations.
#[non_exhaustive]
pub struct ApplicationContext<'a, U, E> {
    /// Serenity's context, like HTTP or cache
    pub discord: &'a serenity::Context,
    /// The interaction which triggered this command execution.
    pub interaction: &'a serenity::ApplicationCommandInteraction,
    /// Keeps track of whether an initial response has been sent.
    ///
    /// Discord requires different HTTP endpoints for initial and additional responses.
    pub has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    /// Read-only reference to the framework
    ///
    /// Useful if you need the list of commands, for example for a custom help command
    pub framework: &'a Framework<U, E>,
    /// The command object which is the current command
    pub command: &'a ApplicationCommand<U, E>,
    /// Your custom user data
    pub data: &'a U,
}
impl<U, E> Clone for ApplicationContext<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U, E> Copy for ApplicationContext<'_, U, E> {}
impl<U, E> crate::_GetGenerics for ApplicationContext<'_, U, E> {
    type U = U;
    type E = E;
}

impl<U, E> ApplicationContext<'_, U, E> {
    /// Defer the response, giving the bot multiple minutes to respond without the user seeing an
    /// "interaction failed error".
    ///
    /// Also sets the [`ApplicationContext::has_sent_initial_response`] flag so the subsequent
    /// response will be sent in the correct manner.
    pub async fn defer_response(&self) -> Result<(), serenity::Error> {
        self.interaction
            .create_interaction_response(self.discord, |f| {
                f.kind(serenity::InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await?;
        self.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

/// Application command specific context to an error in user code
pub struct ApplicationCommandErrorContext<'a, U, E> {
    /// Whether this error occured while running a pre-command check (`true`) or if it happened
    /// in normal command execution (`false`)
    pub while_checking: bool,
    /// Which command was being executed or checked when the error occured
    pub command: &'a ApplicationCommand<U, E>,
    /// Further context
    pub ctx: ApplicationContext<'a, U, E>,
}

impl<U, E> Clone for ApplicationCommandErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        Self {
            while_checking: self.while_checking,
            command: self.command,
            ctx: self.ctx,
        }
    }
}

/// Application command specific configuration of a framework command
pub struct ApplicationCommandOptions<U, E> {
    /// Falls back to the framework-specified value on None. See there for documentation.
    pub on_error: Option<fn(E, ApplicationCommandErrorContext<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(ApplicationContext<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Falls back to the framework-specified value on None. See there for documentation.
    pub defer_response: Option<bool>,
    /// Whether responses to this command should be ephemeral by default.
    pub ephemeral: bool,
    /// Permissions which a user needs to have so that the application command runs.
    pub required_permissions: serenity::Permissions,
    /// If true, only users from the [owners list](crate::FrameworkOptions::owners) may use this
    /// command.
    pub owners_only: bool,
}

impl<U, E> Default for ApplicationCommandOptions<U, E> {
    fn default() -> Self {
        Self {
            on_error: None,
            check: None,
            defer_response: None,
            ephemeral: false,
            required_permissions: serenity::Permissions::empty(),
            owners_only: false,
        }
    }
}

/// Fully defines a slash command in the framework
pub struct SlashCommand<U, E> {
    /// Name of the slash command, displayed in the Discord UI
    pub name: &'static str,
    /// Short description of what the command does, displayed in the Discord UI
    pub description: &'static str,
    /// Vector of builder functions for the parameters
    pub parameters: Vec<
        fn(
            &mut serenity::CreateApplicationCommandOption,
        ) -> &mut serenity::CreateApplicationCommandOption,
    >,
    /// Action which is invoked when the user calls this command
    pub action: for<'a> fn(
        ApplicationContext<'a, U, E>,
        &'a [serenity::ApplicationCommandInteractionDataOption],
    ) -> BoxFuture<'a, Result<(), E>>,
    /// Further configuration
    pub options: ApplicationCommandOptions<U, E>,
}

/// Possible actions that a context menu entry can have
pub enum ContextMenuCommandAction<U, E> {
    /// Context menu entry on a user
    User(fn(ApplicationContext<'_, U, E>, serenity::User) -> BoxFuture<'_, Result<(), E>>),
    /// Context menu entry on a message
    Message(fn(ApplicationContext<'_, U, E>, serenity::Message) -> BoxFuture<'_, Result<(), E>>),
}

/// Fully defines a context menu command in the framework
pub struct ContextMenuCommand<U, E> {
    /// Name of the context menu entry, displayed in the Discord UI
    pub name: &'static str,
    /// Further configuration
    pub options: ApplicationCommandOptions<U, E>,
    /// The target and action of the context menu entry
    pub action: ContextMenuCommandAction<U, E>,
}

/// Defines any application command
pub enum ApplicationCommand<U, E> {
    /// Slash command
    Slash(SlashCommand<U, E>),
    /// Context menu command
    ContextMenu(ContextMenuCommand<U, E>),
}

impl<U, E> ApplicationCommand<U, E> {
    /// If slash command, yield the command name. If context menu command, yield the context
    /// menu entry text.
    pub fn slash_or_context_menu_name(&self) -> &'static str {
        match self {
            Self::Slash(cmd) => cmd.name,
            Self::ContextMenu(cmd) => cmd.name,
        }
    }

    /// Return application command specific configuration
    pub fn options(&self) -> &ApplicationCommandOptions<U, E> {
        match self {
            Self::Slash(cmd) => &cmd.options,
            Self::ContextMenu(cmd) => &cmd.options,
        }
    }

    /// Instruct this application command to register itself in the given builder
    pub fn create<'a>(
        &self,
        interaction: &'a mut serenity::CreateApplicationCommand,
    ) -> &'a mut serenity::CreateApplicationCommand {
        match self {
            Self::Slash(cmd) => {
                interaction.name(cmd.name).description(cmd.description);
                for create_option in &cmd.parameters {
                    let mut option = serenity::CreateApplicationCommandOption::default();
                    create_option(&mut option);
                    interaction.add_option(option);
                }
            }
            Self::ContextMenu(cmd) => {
                interaction.name(cmd.name).kind(match &cmd.action {
                    ContextMenuCommandAction::User(_) => serenity::ApplicationCommandType::User,
                    ContextMenuCommandAction::Message(_) => {
                        serenity::ApplicationCommandType::Message
                    }
                });
            }
        }

        interaction
    }
}

/// Application command specific configuration for the framework
pub struct ApplicationFrameworkOptions<U, E> {
    /// List of bot commands.
    pub commands: Vec<ApplicationCommand<U, E>>,
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// Individual commands may override this callback.
    pub command_check: fn(ApplicationContext<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>,
    /// Whether to send an interaction acknoweldgement.
    ///
    /// Individual commands may override this value.
    ///
    /// If true, send back a quick interaction acknowledgement when receiving an interaction, which
    /// gives your code 15 minutes to respond. When this field is set to false, no acknowledgement
    /// is sent and you need to respond within 3 seconds or the interaction is considered failed by
    /// Discord.
    ///
    /// In some way this is the equivalent of `crate::PrefixFrameworkOptions::broadcast_typing`.
    pub defer_response: bool,
    /// Invoked when a user tries to execute an application command but doesn't have the required
    /// permissions for it.
    ///
    /// This handler should be used to reply with some form of error message. If this handler does
    /// nothing, the user will be shown "Interaction failed" by their Discord client.
    pub missing_permissions_handler: fn(ApplicationContext<'_, U, E>) -> BoxFuture<'_, ()>,
}

impl<U: Send + Sync, E> Default for ApplicationFrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            command_check: |_| Box::pin(async { Ok(true) }),
            defer_response: false,
            missing_permissions_handler: |ctx| {
                Box::pin(async move {
                    let response = format!(
                        "You don't have the required permissions for `/{}`",
                        ctx.command.slash_or_context_menu_name()
                    );
                    let _: Result<_, _> =
                        crate::send_application_reply(ctx, |f| f.content(response).ephemeral(true))
                            .await;
                })
            },
        }
    }
}
