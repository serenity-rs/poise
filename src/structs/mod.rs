//! Plain data structs that define the framework configuration.

mod context;
pub use context::*;

mod framework_options;
pub use framework_options::*;

use crate::{serenity_prelude as serenity, BoxFuture};

// needed for proc macro
#[doc(hidden)]
pub trait _GetGenerics {
    type U;
    type E;
}
impl<U, E> _GetGenerics for Context<'_, U, E> {
    type U = U;
    type E = E;
}

/// Type returned from `#[poise::command]` annotated functions, which contains all of the generated
/// prefix and application commands
#[derive(Default)]
pub struct Command<U, E> {
    // =============
    /// Callback to execute when this command is invoked in a prefix context
    pub prefix_action: Option<
        for<'a> fn(
            crate::PrefixContext<'a, U, E>,
            args: &'a str,
        ) -> BoxFuture<'a, Result<(), crate::FrameworkError<'a, U, E>>>,
    >,
    /// Callback to execute when this command is invoked in a slash context
    pub slash_action: Option<
        for<'a> fn(
            crate::ApplicationContext<'a, U, E>,
            &'a [serenity::ApplicationCommandInteractionDataOption],
        ) -> BoxFuture<'a, Result<(), crate::FrameworkError<'a, U, E>>>,
    >,
    /// Callback to execute when this command is invoked in a context menu context
    ///
    /// The enum variant shows which Discord item this context menu command works on
    pub context_menu_action: Option<crate::ContextMenuCommandAction<U, E>>,

    // ============= Command type agnostic data
    /// Subcommands of this command, if any
    pub subcommands: Vec<Command<U, E>>,
    /// Main name of the command. Aliases (prefix-only) can be set in [`Self::aliases`].
    pub name: &'static str,
    /// A string to identify this particular command within a list of commands.
    ///
    /// Can be configured via the [`crate::command`] macro (though it's probably not needed for most
    /// bots). If not explicitly configured, it falls back to prefix command name, slash command
    /// name, or context menu command name (in that order).
    pub identifying_name: String,
    /// Identifier for the category that this command will be displayed in for help commands.
    pub category: Option<&'static str>,
    /// Whether to hide this command in help menus.
    pub hide_in_help: bool,
    /// Short description of the command. Displayed inline in help menus and similar.
    pub inline_help: Option<&'static str>,
    /// Multiline description with detailed usage instructions. Displayed in the command specific
    /// help: `~help command_name`
    // TODO: fix the inconsistency that this is String and everywhere else it's &'static str
    pub multiline_help: Option<fn() -> String>,
    /// Handles command cooldowns. Mainly for framework internal use
    pub cooldowns: std::sync::Mutex<crate::Cooldowns>,
    /// Permissions which users must have to invoke this command.
    ///
    /// Set to [`serenity::Permissions::empty()`] by default
    pub required_permissions: serenity::Permissions,
    /// Permissions without which command execution will fail. You can set this to fail early and
    /// give a descriptive error message in case the
    /// bot hasn't been assigned the minimum permissions by the guild admin.
    ///
    /// Set to [`serenity::Permissions::empty()`] by default
    pub required_bot_permissions: serenity::Permissions,
    /// If true, only users from the [owners list](crate::FrameworkOptions::owners) may use this
    /// command.
    pub owners_only: bool,
    /// Command-specific override for [`crate::FrameworkOptions::on_error`]
    pub on_error: Option<fn(FrameworkError<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// List of parameters for this command
    ///
    /// Used for registering and parsing slash commands. Can also be used in help commands
    pub parameters: Vec<crate::CommandParameter<U, E>>,

    // ============= Prefix-specific data
    /// Alternative triggers for the command (prefix-only)
    pub aliases: &'static [&'static str],
    /// Whether to enable edit tracking for commands by default (prefix-only)
    ///
    /// Note: only has an effect if `crate::PrefixFrameworkOptions::edit_tracker` is set.
    pub track_edits: bool,
    /// Whether to broadcast a typing indicator while executing this commmand (prefix-only)
    pub broadcast_typing: bool,

    // ============= Application-specific data
    /// Context menu specific name for this command, displayed in Discord's context menu
    pub context_menu_name: Option<&'static str>,
    /// Whether responses to this command should be ephemeral by default (application-only)
    pub ephemeral: bool,
}

impl<U, E> PartialEq for Command<U, E> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl<U, E> Eq for Command<U, E> {}

impl<U, E> std::fmt::Debug for Command<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            prefix_action,
            slash_action,
            context_menu_action,
            subcommands,
            name,
            identifying_name,
            category,
            hide_in_help,
            inline_help,
            multiline_help,
            cooldowns,
            required_permissions,
            required_bot_permissions,
            owners_only,
            on_error,
            check,
            parameters,
            aliases,
            track_edits,
            broadcast_typing,
            context_menu_name,
            ephemeral,
        } = self;

        f.debug_struct("Command")
            .field("prefix_action", &prefix_action.map(|f| f as *const ()))
            .field("slash_action", &slash_action.map(|f| f as *const ()))
            .field("context_menu_action", context_menu_action)
            .field("subcommands", subcommands)
            .field("name", name)
            .field("identifying_name", identifying_name)
            .field("category", category)
            .field("hide_in_help", hide_in_help)
            .field("inline_help", inline_help)
            .field("multiline_help", multiline_help)
            .field("cooldowns", cooldowns)
            .field("required_permissions", required_permissions)
            .field("required_bot_permissions", required_bot_permissions)
            .field("owners_only", owners_only)
            .field("on_error", &on_error.map(|f| f as *const ()))
            .field("check", &check.map(|f| f as *const ()))
            .field("parameters", parameters)
            .field("aliases", aliases)
            .field("track_edits", track_edits)
            .field("broadcast_typing", broadcast_typing)
            .field("context_menu_name", context_menu_name)
            .field("ephemeral", ephemeral)
            .finish()
    }
}

impl<U, E> Command<U, E> {
    fn create_as_subcommand(&self) -> Option<serenity::CreateApplicationCommandOption> {
        self.slash_action?;

        let mut builder = serenity::CreateApplicationCommandOption::default();
        builder
            .name(self.name)
            .description(self.inline_help.unwrap_or("A slash command"));

        if self.subcommands.is_empty() {
            builder.kind(serenity::ApplicationCommandOptionType::SubCommand);

            for param in &self.parameters {
                // Using `?` because if this command has slash-incompatible parameters, we cannot
                // just ignore them but have to abort the creation process entirely
                builder.add_sub_option(param.create_as_slash_command_option()?);
            }
        } else {
            builder.kind(serenity::ApplicationCommandOptionType::SubCommandGroup);

            for subcommand in &self.subcommands {
                if let Some(subcommand) = subcommand.create_as_subcommand() {
                    builder.add_sub_option(subcommand);
                }
            }
        }

        Some(builder)
    }

    /// Generates a slash command builder from this [`Command`] instance. This can be used
    /// to register this command on Discord's servers
    pub fn create_as_slash_command(&self) -> Option<serenity::CreateApplicationCommand> {
        self.slash_action?;

        let mut builder = serenity::CreateApplicationCommand::default();
        builder
            .name(self.name)
            .description(self.inline_help.unwrap_or("A slash command"));

        if self.subcommands.is_empty() {
            for param in &self.parameters {
                // Using `?` because if this command has slash-incompatible parameters, we cannot
                // just ignore them but have to abort the creation process entirely
                builder.add_option(param.create_as_slash_command_option()?);
            }
        } else {
            for subcommand in &self.subcommands {
                if let Some(subcommand) = subcommand.create_as_subcommand() {
                    builder.add_option(subcommand);
                }
            }
        }

        Some(builder)
    }

    /// Generates a context menu command builder from this [`Command`] instance. This can be used
    /// to register this command on Discord's servers
    pub fn create_as_context_menu_command(&self) -> Option<serenity::CreateApplicationCommand> {
        let context_menu_action = self.context_menu_action?;

        let mut builder = serenity::CreateApplicationCommand::default();
        builder
            .name(self.context_menu_name.unwrap_or(self.name))
            .kind(match context_menu_action {
                crate::ContextMenuCommandAction::User(_) => serenity::ApplicationCommandType::User,
                crate::ContextMenuCommandAction::Message(_) => {
                    serenity::ApplicationCommandType::Message
                }
            });

        Some(builder)
    }

    /// **Deprecated**
    #[deprecated = "Please use `crate::Command { category: \"...\", ..command() }` instead"]
    pub fn category(&mut self, category: &'static str) -> &mut Self {
        self.category = Some(category);
        self
    }

    /// Insert a subcommand
    pub fn subcommand(
        &mut self,
        mut subcommand: crate::Command<U, E>,
        meta_builder: impl FnOnce(&mut Self) -> &mut Self,
    ) -> &mut Self {
        meta_builder(&mut subcommand);
        self.subcommands.push(subcommand);
        self
    }
}

/// Any error that can occur while the bot runs. Either thrown by user code (those variants will
/// have an `error` field with your error type `E` in it), or originating from within the framework.
///
/// These errors are handled with the [`crate::FrameworkOptions::on_error`] callback
#[derive(Debug)]
pub enum FrameworkError<'a, U, E> {
    /// User code threw an error in user data setup
    Setup {
        /// Error which was thrown in the setup code
        error: E,
    },
    /// User code threw an error in generic event listener
    Listener {
        /// Error which was thrown in the listener code
        error: E,
        /// Which event was being processed when the error occurred
        event: &'a crate::Event<'a>,
    },
    /// Error occured during command execution
    Command {
        /// Error which was thrown in the command code
        error: E,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// A command argument failed to parse from the Discord message or interaction content
    ArgumentParse {
        /// Error which was thrown by the parameter type's parsing routine
        error: Box<dyn std::error::Error + Send + Sync>,
        /// If applicable, the input on which parsing failed
        input: Option<String>,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Expected a certain argument type at a certain position in the unstructured list of
    /// arguments, but found something else.
    ///
    /// Most often the result of the bot not having registered the command in Discord, so Discord
    /// stores an outdated version of the command and its parameters.
    CommandStructureMismatch {
        /// Developer-readable description of the type mismatch
        description: &'static str,
        /// General context
        ctx: crate::ApplicationContext<'a, U, E>,
    },
    /// Command was invoked before its cooldown expired
    CooldownHit {
        /// Time until the command may be invoked for the next time in the given context
        remaining_cooldown: std::time::Duration,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the bot is lacking the permissions specified in
    /// [`crate::Command::required_bot_permissions`]
    MissingBotPermissions {
        /// Which permissions in particular the bot is lacking for this command
        missing_permissions: serenity::Permissions,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the user is lacking the permissions specified in
    /// [`crate::Command::required_bot_permissions`]
    MissingUserPermissions {
        /// List of permissions that the user is lacking. May be None if retrieving the user's
        /// permissions failed
        missing_permissions: Option<serenity::Permissions>,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// A non-owner tried to invoke an owners-only command
    NotAnOwner {
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Provided pre-command check either errored, or returned false, so command execution aborted
    CommandCheckFailed {
        /// If execution wasn't aborted because of an error but because it successfully returned
        /// false, this field is None
        error: Option<E>,
        /// General context
        ctx: Context<'a, U, E>,
    },
}
