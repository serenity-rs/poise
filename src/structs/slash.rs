//! Holds application command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture};

/// Application command specific context passed to command invocations.
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub struct ApplicationContext<'a, U, E> {
    /// Serenity's context, like HTTP or cache
    #[derivative(Debug = "ignore")]
    pub discord: &'a serenity::Context,
    /// The interaction which triggered this command execution.
    pub interaction: crate::ApplicationCommandOrAutocompleteInteraction<'a>,
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
    /// The command object which is the current command
    pub command: &'a crate::Command<U, E>,
    /// Your custom user data
    // TODO: redundant with framework
    #[derivative(Debug = "ignore")]
    pub data: &'a U,
    /// Custom user data carried across a single command invocation
    pub invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    // #[non_exhaustive] forbids struct update syntax for ?? reason
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
        let interaction = match self.interaction {
            crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
            crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => return Ok(()),
        };

        if !self
            .has_sent_initial_response
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            interaction
                .create_interaction_response(
                    self.discord,
                    serenity::CreateInteractionResponse::Defer(
                        serenity::CreateInteractionResponseMessage::default().ephemeral(ephemeral),
                    ),
                )
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
}
impl<U, E> Copy for ContextMenuCommandAction<U, E> {}
impl<U, E> Clone for ContextMenuCommandAction<U, E> {
    fn clone(&self) -> Self {
        *self
    }
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
    pub choices: Vec<crate::CommandParameterChoice>,
    /// Closure that sets this parameter's type and min/max value in the given builder
    ///
    /// For example a u32 [`CommandParameter`] would store this as the [`Self::type_setter`]:
    /// ```rust
    /// # use poise::serenity_prelude as serenity;
    /// # let _: fn(serenity::CreateApplicationCommandOption) -> serenity::CreateApplicationCommandOption =
    /// |b| b.kind(serenity::CommandOptionType::Integer).min_int_value(0).max_int_value(u64::MAX)
    /// # ;
    /// ```
    #[derivative(Debug = "ignore")]
    pub type_setter: Option<fn(serenity::CommandOption) -> serenity::CommandOption>,
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
}

impl<U, E> CommandParameter<U, E> {
    /// Generates a slash command parameter builder from this [`CommandParameter`] instance. This
    /// can be used to register the command on Discord's servers
    pub fn create_as_slash_command_option(&self) -> Option<serenity::CreateCommandOption> {
        let mut b = serenity::CreateCommandOption::new(
            serenity::CommandOptionType::Unknown(0), // Will be overwritten by type_setter below
            &self.name,
            self.description
                .as_deref()
                .unwrap_or("A slash command parameter"),
        );

        b = b
            .required(self.required)
            .set_autocomplete(self.autocomplete_callback.is_some());
        for (locale, name) in &self.name_localizations {
            b = b.name_localized(locale, name);
        }
        for (locale, description) in &self.description_localizations {
            b = b.description_localized(locale, description);
        }
        if let Some(channel_types) = &self.channel_types {
            b = b.channel_types(channel_types.clone());
        }
        for (i, choice) in self.choices.iter().enumerate() {
            b = b.add_int_choice_localized(&choice.name, i as _, choice.localizations.iter());
        }
        b = (self.type_setter?)(b);
        Some(b)
    }
}
