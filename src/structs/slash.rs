//! Holds application command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture};

/// Specifies if the current invokation is from a Command or Autocomplete.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandInteractionType {
    /// Invoked from an application command
    Command,
    /// Invoked from an autocomplete interaction
    Autocomplete,
}

/// Application command specific context passed to command invocations.
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub struct ApplicationContext<'a, U, E> {
    /// Serenity's context, like HTTP or cache
    #[derivative(Debug = "ignore")]
    pub serenity_context: &'a serenity::Context,
    /// The interaction which triggered this command execution.
    pub interaction: &'a serenity::CommandInteraction,
    /// The type of the interaction which triggered this command execution.
    pub interaction_type: CommandInteractionType,
    /// Slash command arguments
    ///
    /// **Not** equivalent to `self.interaction.data().options`. That one refers to just the
    /// top-level command arguments, whereas [`Self::args`] is the options of the actual
    /// subcommand, if any.
    pub args: &'a [serenity::ResolvedOption<'a>],
    /// Keeps track of whether an initial response has been sent.
    ///
    /// Discord requires different HTTP endpoints for initial and additional responses.
    pub has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    /// Read-only reference to the framework
    ///
    /// Useful if you need the list of commands, for example for a custom help command
    #[derivative(Debug = "ignore")]
    pub framework: crate::FrameworkContext<'a, U, E>,
    /// If the invoked command was a subcommand, these are the parent commands, ordered top down.
    pub parent_commands: &'a [&'a crate::Command<U, E>],
    /// The command object which is the current command
    pub command: &'a crate::Command<U, E>,
    /// Your custom user data
    // TODO: redundant with framework
    #[derivative(Debug = "ignore")]
    pub data: &'a U,
    /// Custom user data carried across a single command invocation
    pub invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    // #[non_exhaustive] forbids struct update syntax for ?? reason
    #[cfg(any(not(feature = "unstable_exhaustive_types"), doc))]
    #[doc(hidden)]
    pub __non_exhaustive: (),
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
    /// See [`crate::Context::defer()`]
    pub async fn defer_response(&self, ephemeral: bool) -> Result<(), serenity::Error> {
        if !self
            .has_sent_initial_response
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            let response = serenity::CreateInteractionResponse::Defer(
                serenity::CreateInteractionResponseMessage::new().ephemeral(ephemeral),
            );

            self.interaction
                .create_response(self.serenity_context, response)
                .await?;

            self.has_sent_initial_response
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
}

/// Possible actions that a context menu entry can have
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub enum ContextMenuCommandAction<U, E> {
    /// Context menu entry on a user
    User(
        #[derivative(Debug = "ignore")]
        fn(
            ApplicationContext<'_, U, E>,
            serenity::User,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ),
    /// Context menu entry on a message
    Message(
        #[derivative(Debug = "ignore")]
        fn(
            ApplicationContext<'_, U, E>,
            serenity::Message,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ),
    #[doc(hidden)]
    __NonExhaustive,
}
impl<U, E> Copy for ContextMenuCommandAction<U, E> {}
impl<U, E> Clone for ContextMenuCommandAction<U, E> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A single drop-down choice in a slash command choice parameter
#[derive(Debug, Clone)]
pub struct CommandParameterChoice {
    /// Label of this choice
    pub name: String,
    /// Localized labels with locale string as the key (slash-only)
    pub localizations: std::collections::HashMap<String, String>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

/// A single parameter of a [`crate::Command`]
#[derive(Clone, derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub struct CommandParameter<U, E> {
    /// Name of this command parameter
    pub name: String,
    /// Localized names with locale string as the key (slash-only)
    pub name_localizations: std::collections::HashMap<String, String>,
    /// Description of the command. Required for slash commands
    pub description: Option<String>,
    /// Localized descriptions with locale string as the key (slash-only)
    pub description_localizations: std::collections::HashMap<String, String>,
    /// `true` is this parameter is required, `false` if it's optional or variadic
    pub required: bool,
    /// If this parameter is a channel, users can only enter these channel types in a slash command
    ///
    /// Prefix commands are currently unaffected by this
    pub channel_types: Option<Vec<serenity::ChannelType>>,
    /// If this parameter is a choice parameter, this is the fixed list of options
    pub choices: Vec<CommandParameterChoice>,
    /// Closure that sets this parameter's type and min/max value in the given builder
    ///
    /// For example a u32 [`CommandParameter`] would store this as the [`Self::type_setter`]:
    /// ```rust
    /// # use poise::serenity_prelude as serenity;
    /// # let _: fn(serenity::CreateCommandOption) -> serenity::CreateCommandOption =
    /// |b| b.kind(serenity::CommandOptionType::Integer).min_int_value(0).max_int_value(u64::MAX)
    /// # ;
    /// ```
    #[derivative(Debug = "ignore")]
    pub type_setter: Option<fn(serenity::CreateCommandOption) -> serenity::CreateCommandOption>,
    /// Optionally, a callback that is invoked on autocomplete interactions. This closure should
    /// extract the partial argument from the given JSON value and generate the autocomplete
    /// response which contains the list of autocomplete suggestions.
    #[derivative(Debug = "ignore")]
    pub autocomplete_callback: Option<
        for<'a> fn(
            crate::ApplicationContext<'a, U, E>,
            &'a str,
        ) -> BoxFuture<
            'a,
            Result<serenity::CreateAutocompleteResponse, crate::SlashArgError>,
        >,
    >,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl<U, E> CommandParameter<U, E> {
    /// Generates a slash command parameter builder from this [`CommandParameter`] instance. This
    /// can be used to register the command on Discord's servers
    pub fn create_as_slash_command_option(&self) -> Option<serenity::CreateCommandOption> {
        let description = self
            .description
            .as_deref()
            .unwrap_or("A slash command parameter");

        let mut builder = serenity::CreateCommandOption::new(
            serenity::CommandOptionType::String,
            self.name.clone(),
            description,
        );

        builder = builder
            .required(self.required)
            .set_autocomplete(self.autocomplete_callback.is_some());

        for (locale, name) in &self.name_localizations {
            builder = builder.name_localized(locale, name);
        }
        for (locale, description) in &self.description_localizations {
            builder = builder.description_localized(locale, description);
        }
        if let Some(channel_types) = self.channel_types.clone() {
            builder = builder.channel_types(channel_types);
        }
        for (i, choice) in self.choices.iter().enumerate() {
            builder =
                builder.add_int_choice_localized(&choice.name, i as _, choice.localizations.iter());
        }

        Some((self.type_setter?)(builder))
    }
}
