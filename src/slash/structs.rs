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
    pub command: ApplicationCommand<'a, U, E>,
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
    pub async fn defer_response(&self, ephemeral: bool) -> Result<(), serenity::Error> {
        let mut flags = serenity::InteractionApplicationCommandCallbackDataFlags::empty();
        if ephemeral {
            flags |= serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL;
        }

        self.interaction
            .create_interaction_response(self.discord, |f| {
                f.kind(serenity::InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|f| f.flags(flags))
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
    pub command: ApplicationCommand<'a, U, E>,
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
            ephemeral: false,
            required_permissions: serenity::Permissions::empty(),
            owners_only: false,
        }
    }
}

/// Fully defines a single slash command in the framework
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

/// A single slash command or slash command group
pub enum SlashCommandMeta<U, E> {
    /// Single slash command
    Command(SlashCommand<U, E>),
    /// Command group with a list of subcommands
    CommandGroup {
        /// Name of the command group, i.e. the identifier preceding subcommands
        name: &'static str,
        /// Description of the command group (currently not visible in Discord UI)
        description: &'static str,
        /// List of command group subcommands
        subcommands: Vec<SlashCommandMeta<U, E>>,
    },
}

impl<U, E> SlashCommandMeta<U, E> {
    fn create_as_subcommand<'a>(
        &self,
        builder: &'a mut serenity::CreateApplicationCommandOption,
    ) -> &'a mut serenity::CreateApplicationCommandOption {
        match self {
            Self::CommandGroup {
                name,
                description,
                subcommands,
            } => {
                builder.kind(serenity::ApplicationCommandOptionType::SubCommandGroup);
                builder.name(name).description(description);

                for sub_subcommand in subcommands {
                    builder.create_sub_option(|f| sub_subcommand.create_as_subcommand(f));
                }
            }
            Self::Command(command) => {
                builder.kind(serenity::ApplicationCommandOptionType::SubCommand);
                builder.name(command.name).description(command.description);

                for create_option in &command.parameters {
                    let mut option = serenity::CreateApplicationCommandOption::default();
                    create_option(&mut option);
                    builder.add_sub_option(option);
                }
            }
        }
        builder
    }

    fn create<'a>(
        &self,
        interaction: &'a mut serenity::CreateApplicationCommand,
    ) -> &'a mut serenity::CreateApplicationCommand {
        match self {
            Self::CommandGroup {
                name,
                description,
                subcommands,
            } => {
                interaction.name(name).description(description);

                for subcommand in subcommands {
                    interaction.create_option(|f| subcommand.create_as_subcommand(f));
                }
            }
            Self::Command(command) => {
                interaction
                    .name(command.name)
                    .description(command.description);

                for create_option in &command.parameters {
                    let mut option = serenity::CreateApplicationCommandOption::default();
                    create_option(&mut option);
                    interaction.add_option(option);
                }
            }
        }
        interaction
    }
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

/// Defines any application command, including subcommands if supported by the application command
/// type
pub enum ApplicationCommandTree<U, E> {
    /// Slash command
    Slash(SlashCommandMeta<U, E>),
    /// Context menu command
    ContextMenu(ContextMenuCommand<U, E>),
}

impl<U, E> ApplicationCommandTree<U, E> {
    /// Instruct this application command to register itself in the given builder
    pub fn create<'b>(
        &self,
        interaction: &'b mut serenity::CreateApplicationCommand,
    ) -> &'b mut serenity::CreateApplicationCommand {
        match self {
            Self::Slash(cmd) => cmd.create(interaction),
            Self::ContextMenu(cmd) => interaction.name(cmd.name).kind(match &cmd.action {
                ContextMenuCommandAction::User(_) => serenity::ApplicationCommandType::User,
                ContextMenuCommandAction::Message(_) => serenity::ApplicationCommandType::Message,
            }),
        }
    }
}

/// A view into a leaf of an application command tree. **Not an owned type!**
pub enum ApplicationCommand<'a, U, E> {
    /// Slash command
    Slash(&'a SlashCommand<U, E>),
    /// Context menu command
    ContextMenu(&'a ContextMenuCommand<U, E>),
}
impl<U, E> Copy for ApplicationCommand<'_, U, E> {}
impl<U, E> Clone for ApplicationCommand<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, U, E> ApplicationCommand<'a, U, E> {
    /// If slash command, yield the command name. If context menu command, yield the context
    /// menu entry text.
    pub fn slash_or_context_menu_name(self) -> &'static str {
        match self {
            Self::Slash(cmd) => cmd.name,
            Self::ContextMenu(cmd) => cmd.name,
        }
    }

    /// Return application command specific configuration
    pub fn options(self) -> &'a ApplicationCommandOptions<U, E> {
        match self {
            Self::Slash(cmd) => &cmd.options,
            Self::ContextMenu(cmd) => &cmd.options,
        }
    }
}

/// Application command specific configuration for the framework
pub struct ApplicationFrameworkOptions<U, E> {
    /// List of bot commands.
    pub commands: Vec<ApplicationCommandTree<U, E>>,
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
