//! The Command struct, which stores all information about a single framework command

use crate::{serenity_prelude as serenity, BoxFuture};

/// Type returned from `#[poise::command]` annotated functions, which contains all of the generated
/// prefix and application commands
#[derive(derivative::Derivative)]
#[derivative(Default, Debug(bound = ""))]
pub struct Command<U, E> {
    // =============
    /// Callback to execute when this command is invoked in a prefix context
    #[derivative(Debug = "ignore")]
    pub prefix_action: Option<
        for<'a> fn(
            crate::PrefixContext<'a, U, E>,
        ) -> BoxFuture<'a, Result<(), crate::FrameworkError<'a, U, E>>>,
    >,
    /// Callback to execute when this command is invoked in a slash context
    #[derivative(Debug = "ignore")]
    pub slash_action: Option<
        for<'a> fn(
            crate::ApplicationContext<'a, U, E>,
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
    pub name: String,
    /// Localized names with locale string as the key (slash-only)
    pub name_localizations: std::collections::HashMap<String, String>,
    /// Full name including parent command names.
    ///
    /// Initially set to just [`Self::name`] and properly populated when the framework is started.
    pub qualified_name: String,
    /// A string to identify this particular command within a list of commands.
    ///
    /// Can be configured via the [`crate::command`] macro (though it's probably not needed for most
    /// bots). If not explicitly configured, it falls back to the command function name.
    pub identifying_name: String,
    /// Identifier for the category that this command will be displayed in for help commands.
    pub category: Option<&'static str>,
    /// Whether to hide this command in help menus.
    pub hide_in_help: bool,
    /// Short description of the command. Displayed inline in help menus and similar.
    // TODO: rename to description
    pub description: Option<String>,
    /// Localized descriptions with locale string as the key (slash-only)
    pub description_localizations: std::collections::HashMap<String, String>,
    /// Multiline description with detailed usage instructions. Displayed in the command specific
    /// help: `~help command_name`
    // TODO: fix the inconsistency that this is String and everywhere else it's &'static str
    pub help_text: Option<fn() -> String>,
    /// Handles command cooldowns. Mainly for framework internal use
    pub cooldowns: std::sync::Mutex<crate::Cooldowns>,
    /// After the first response, whether to post subsequent responses as edits to the initial
    /// message
    ///
    /// Note: in prefix commands, this only has an effect if
    /// `crate::PrefixFrameworkOptions::edit_tracker` is set.
    pub reuse_response: bool,
    /// Permissions which users must have to invoke this command. Used by Discord to set who can
    /// invoke this as a slash command. Not used on prefix commands or checked internally.
    ///
    /// Set to [`serenity::Permissions::empty()`] by default
    pub default_member_permissions: serenity::Permissions,
    /// Permissions which users must have to invoke this command. This is checked internally and
    /// works for both prefix commands and slash commands.
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
    /// If true, only people in guilds may use this command
    pub guild_only: bool,
    /// If true, the command may only run in DMs
    pub dm_only: bool,
    /// If true, the command may only run in NSFW channels
    pub nsfw_only: bool,
    /// Command-specific override for [`crate::FrameworkOptions::on_error`]
    #[derivative(Debug = "ignore")]
    pub on_error: Option<fn(crate::FrameworkError<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If any of these functions returns false, this command will not be executed.
    #[derivative(Debug = "ignore")]
    pub checks: Vec<fn(crate::Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// List of parameters for this command
    ///
    /// Used for registering and parsing slash commands. Can also be used in help commands
    pub parameters: Vec<crate::CommandParameter<U, E>>,
    /// Arbitrary data, useful for storing custom metadata about your commands
    #[derivative(Default(value = "Box::new(())"))]
    pub custom_data: Box<dyn std::any::Any + Send + Sync>,

    // ============= Prefix-specific data
    /// Alternative triggers for the command (prefix-only)
    pub aliases: &'static [&'static str],
    /// Whether to rerun the command if an existing invocation message is edited (prefix-only)
    pub invoke_on_edit: bool,
    /// Whether to broadcast a typing indicator while executing this commmand (prefix-only)
    pub broadcast_typing: bool,

    // ============= Application-specific data
    /// Context menu specific name for this command, displayed in Discord's context menu
    pub context_menu_name: Option<&'static str>,
    /// Whether responses to this command should be ephemeral by default (application-only)
    pub ephemeral: bool,

    // Like #[non_exhaustive], but #[poise::command] still needs to be able to create an instance
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl<U, E> PartialEq for Command<U, E> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
impl<U, E> Eq for Command<U, E> {}

impl<U, E> Command<U, E> {
    /// Serializes this Command into an application command option, which is the form which Discord
    /// requires subcommands to be in
    fn create_as_subcommand(&self) -> Option<serenity::CreateApplicationCommandOption> {
        self.slash_action?;

        let mut builder = serenity::CreateApplicationCommandOption::default();
        builder
            .name(&self.name)
            .description(self.description.as_deref().unwrap_or("A slash command"));
        for (locale, name) in &self.name_localizations {
            builder.name_localized(locale, name);
        }
        for (locale, description) in &self.description_localizations {
            builder.description_localized(locale, description);
        }

        if self.subcommands.is_empty() {
            builder.kind(serenity::CommandOptionType::SubCommand);

            for param in &self.parameters {
                // Using `?` because if this command has slash-incompatible parameters, we cannot
                // just ignore them but have to abort the creation process entirely
                builder.add_sub_option(param.create_as_slash_command_option()?);
            }
        } else {
            builder.kind(serenity::CommandOptionType::SubCommandGroup);

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
            .name(&self.name)
            .description(self.description.as_deref().unwrap_or("A slash command"));
        for (locale, name) in &self.name_localizations {
            builder.name_localized(locale, name);
        }
        for (locale, description) in &self.description_localizations {
            builder.description_localized(locale, description);
        }

        // This is_empty check is needed because Discord special cases empty
        // default_member_permissions to mean "admin-only" (yes it's stupid)
        if !self.default_member_permissions.is_empty() {
            builder.default_member_permissions(self.default_member_permissions);
        }

        if self.guild_only {
            builder.dm_permission(false);
        }

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
            // TODO: localization?
            .name(self.context_menu_name.unwrap_or(&self.name))
            .kind(match context_menu_action {
                crate::ContextMenuCommandAction::User(_) => serenity::CommandType::User,
                crate::ContextMenuCommandAction::Message(_) => serenity::CommandType::Message,
            });

        if self.guild_only {
            builder.dm_permission(false);
        }

        Some(builder)
    }

    /// **Deprecated**
    #[deprecated = "Please use `poise::Command { category: \"...\", ..command() }` instead"]
    pub fn category(&mut self, category: &'static str) -> &mut Self {
        self.category = Some(category);
        self
    }

    /// Insert a subcommand
    #[deprecated = "Please use `poise::Command { subcommands: vec![...], ..command() }` instead"]
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
