//! Holds application command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture, Framework};

/// Abstracts over a refernce to an application command interaction or autocomplete interaction
#[derive(Copy, Clone, Debug)]
pub enum ApplicationCommandOrAutocompleteInteraction<'a> {
    /// An application command interaction
    ApplicationCommand(&'a serenity::ApplicationCommandInteraction),
    /// An autocomplete interaction
    Autocomplete(&'a serenity::AutocompleteInteraction),
}

impl<'a> ApplicationCommandOrAutocompleteInteraction<'a> {
    /// Returns the data field of the underlying interaction
    pub fn data(self) -> &'a serenity::ApplicationCommandInteractionData {
        match self {
            Self::ApplicationCommand(x) => &x.data,
            Self::Autocomplete(x) => &x.data,
        }
    }

    /// Returns the ID of the underlying interaction
    pub fn id(self) -> serenity::InteractionId {
        match self {
            Self::ApplicationCommand(x) => x.id,
            Self::Autocomplete(x) => x.id,
        }
    }

    /// Returns the guild ID of the underlying interaction
    pub fn guild_id(self) -> Option<serenity::GuildId> {
        match self {
            Self::ApplicationCommand(x) => x.guild_id,
            Self::Autocomplete(x) => x.guild_id,
        }
    }

    /// Returns the channel ID of the underlying interaction
    pub fn channel_id(self) -> serenity::ChannelId {
        match self {
            Self::ApplicationCommand(x) => x.channel_id,
            Self::Autocomplete(x) => x.channel_id,
        }
    }

    /// Returns the member field of the underlying interaction
    pub fn member(self) -> Option<&'a serenity::Member> {
        match self {
            Self::ApplicationCommand(x) => x.member.as_ref(),
            Self::Autocomplete(x) => x.member.as_ref(),
        }
    }

    /// Returns the user field of the underlying interaction
    pub fn user(self) -> &'a serenity::User {
        match self {
            Self::ApplicationCommand(x) => &x.user,
            Self::Autocomplete(x) => &x.user,
        }
    }
}

/// Application command specific context passed to command invocations.
pub struct ApplicationContext<'a, U, E> {
    /// Serenity's context, like HTTP or cache
    pub discord: &'a serenity::Context,
    /// The interaction which triggered this command execution.
    pub interaction: ApplicationCommandOrAutocompleteInteraction<'a>,
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

impl<U: std::fmt::Debug, E: std::fmt::Debug> std::fmt::Debug for ApplicationContext<'_, U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            discord,
            interaction,
            has_sent_initial_response,
            framework,
            command,
            data,
        } = self;

        f.debug_struct("ApplicationContext")
            .field("discord", &"<serenity Context>")
            .field("interaction", interaction)
            .field("has_sent_initial_response", has_sent_initial_response)
            .field("framework", &"<poise Framework>")
            .field("command", command)
            .field("data", data)
            .finish()
    }
}

impl<U, E> ApplicationContext<'_, U, E> {
    /// See [`crate::Context::defer()`]
    pub async fn defer_response(&self, ephemeral: bool) -> Result<(), serenity::Error> {
        let interaction = match self.interaction {
            ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
            ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => return Ok(()),
        };

        let mut flags = serenity::InteractionApplicationCommandCallbackDataFlags::empty();
        if ephemeral {
            flags |= serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL;
        }

        interaction
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

/// Application command specific configuration of a framework command
#[derive(Clone, Debug)]
pub struct ApplicationCommandOptions {
    /// Whether responses to this command should be ephemeral by default.
    pub ephemeral: bool,
}

#[allow(clippy::derivable_impls)] // the default values have important meaning
impl Default for ApplicationCommandOptions {
    fn default() -> Self {
        Self { ephemeral: false }
    }
}

/// A single parameter of a slash command
#[derive(Clone)]
pub struct SlashCommandParameter<U, E> {
    /// Builder function for this parameters
    pub builder: fn(
        &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption,
    /// Optionally, a callback on autocomplete interactions. If the focused option in the
    /// autocomplete interaction matches this parameter, an autocomplete response should be sent
    pub autocomplete_callback: Option<
        for<'a> fn(
            crate::ApplicationContext<'a, U, E>,
            &'a serenity::AutocompleteInteraction,
            &'a [serenity::ApplicationCommandInteractionDataOption],
        ) -> BoxFuture<'a, Result<(), E>>,
    >,
}

impl<U, E> std::fmt::Debug for SlashCommandParameter<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            builder,
            autocomplete_callback,
        } = self;

        f.debug_struct("SlashCommandParameter")
            .field("builder", &(*builder as *const ()))
            .field(
                "autocomplete_callback",
                &autocomplete_callback.map(|f| f as *const ()),
            )
            .finish()
    }
}

/// Fully defines a single slash command in the framework
#[derive(Clone)]
pub struct SlashCommand<U, E> {
    /// Name of the slash command, displayed in the Discord UI
    pub name: &'static str,
    /// Short description of what the command does, displayed in the Discord UI
    pub description: &'static str,
    /// List of parameters for this slash command
    pub parameters: Vec<SlashCommandParameter<U, E>>,
    /// Action which is invoked when the user calls this command
    pub action: for<'a> fn(
        ApplicationContext<'a, U, E>,
        &'a [serenity::ApplicationCommandInteractionDataOption],
    ) -> BoxFuture<'a, Result<(), E>>,
    /// The command ID, shared across all command types that belong to the same implementation
    pub id: std::sync::Arc<crate::CommandId<U, E>>,
    /// Further configuration
    pub options: ApplicationCommandOptions,
}

impl<U, E> std::fmt::Debug for SlashCommand<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            description,
            parameters,
            action,
            id,
            options,
        } = self;

        f.debug_struct("SlashCommand")
            .field("name", name)
            .field("description", description)
            .field("parameters", parameters)
            .field("action", &(*action as *const ()))
            .field("id", id)
            .field("options", options)
            .finish()
    }
}

/// A single slash command or slash command group
#[derive(Clone, Debug)]
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
        /// Contains command-type agnostic data
        id: std::sync::Arc<crate::CommandId<U, E>>,
    },
}

impl<U, E> SlashCommandMeta<U, E> {
    /// Returns the name of this command or command group
    pub fn name(&self) -> &'static str {
        match self {
            Self::Command(cmd) => cmd.name,
            Self::CommandGroup { name, .. } => name,
        }
    }

    /// Returns the description of this command or command group
    pub fn description(&self) -> &'static str {
        match self {
            Self::Command(cmd) => cmd.description,
            Self::CommandGroup { description, .. } => description,
        }
    }

    /// Returns the [`crate::CommandId`] for this command or command group
    pub fn id(&self) -> &std::sync::Arc<crate::CommandId<U, E>> {
        match self {
            SlashCommandMeta::Command(cmd) => &cmd.id,
            SlashCommandMeta::CommandGroup { id, .. } => id,
        }
    }

    fn create_as_subcommand<'a>(
        &self,
        builder: &'a mut serenity::CreateApplicationCommandOption,
    ) -> &'a mut serenity::CreateApplicationCommandOption {
        match self {
            Self::CommandGroup {
                name,
                description,
                subcommands,
                id: _,
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

                for param in &command.parameters {
                    let mut option = serenity::CreateApplicationCommandOption::default();
                    (param.builder)(&mut option);
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
                id: _,
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

                for param in &command.parameters {
                    let mut option = serenity::CreateApplicationCommandOption::default();
                    (param.builder)(&mut option);
                    interaction.add_option(option);
                }
            }
        }
        interaction
    }
}

/// Possible actions that a context menu entry can have
#[derive(Clone)]
pub enum ContextMenuCommandAction<U, E> {
    /// Context menu entry on a user
    User(fn(ApplicationContext<'_, U, E>, serenity::User) -> BoxFuture<'_, Result<(), E>>),
    /// Context menu entry on a message
    Message(fn(ApplicationContext<'_, U, E>, serenity::Message) -> BoxFuture<'_, Result<(), E>>),
}

impl<U, E> std::fmt::Debug for ContextMenuCommandAction<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::User(x) => f.debug_tuple("User").field(&(x as *const ())).finish(),
            Self::Message(x) => f.debug_tuple("Message").field(&(x as *const ())).finish(),
        }
    }
}

/// Fully defines a context menu command in the framework
#[derive(Clone, Debug)]
pub struct ContextMenuCommand<U, E> {
    /// Name of the context menu entry, displayed in the Discord UI
    pub name: &'static str,
    /// Further configuration
    pub options: ApplicationCommandOptions,
    /// The command ID, shared across all command types that belong to the same implementation
    pub id: std::sync::Arc<crate::CommandId<U, E>>,
    /// The target and action of the context menu entry
    pub action: ContextMenuCommandAction<U, E>,
}

/// Defines any application command, including subcommands if supported by the application command
/// type
#[derive(Clone, Debug)]
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
#[derive(Debug)]
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
    pub fn options(self) -> &'a ApplicationCommandOptions {
        match self {
            Self::Slash(cmd) => &cmd.options,
            Self::ContextMenu(cmd) => &cmd.options,
        }
    }

    /// Return the command ID, shared across all command types that belong to the same
    /// implementation
    pub fn id(self) -> &'a std::sync::Arc<crate::CommandId<U, E>> {
        match self {
            Self::Slash(cmd) => &cmd.id,
            Self::ContextMenu(cmd) => &cmd.id,
        }
    }
}

/// Application command specific configuration for the framework
#[derive(Clone, Debug)]
pub struct ApplicationFrameworkOptions<U, E> {
    /// List of bot commands.
    pub commands: Vec<ApplicationCommandTree<U, E>>,
}

impl<U, E> Default for ApplicationFrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}
