//! Holds application command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture};

/// Abstracts over a refernce to an application command interaction or autocomplete interaction
///
/// Used in [`crate::ApplicationContext`]. We need to support autocomplete interactions in
/// [`crate::ApplicationContext`] because command checks are invoked for autocomplete interactions
/// too: we don't want poise accidentally leaking sensitive information through autocomplete
/// suggestions
// TODO: inline this struct once merged into main branch
#[derive(Copy, Clone, Debug)]
pub enum CommandOrAutocompleteInteraction<'a> {
    /// An application command interaction
    Command(&'a serenity::CommandInteraction),
    /// An autocomplete interaction
    Autocomplete(&'a serenity::CommandInteraction),
}

impl<'a> CommandOrAutocompleteInteraction<'a> {
    /// Returns the data field of the underlying interaction
    pub fn data(self) -> &'a serenity::CommandData {
        match self {
            Self::Command(x) => &x.data,
            Self::Autocomplete(x) => &x.data,
        }
    }

    /// Returns the ID of the underlying interaction
    pub fn id(self) -> serenity::InteractionId {
        match self {
            Self::Command(x) => x.id,
            Self::Autocomplete(x) => x.id,
        }
    }

    /// Returns the guild ID of the underlying interaction
    pub fn guild_id(self) -> Option<serenity::GuildId> {
        match self {
            Self::Command(x) => x.guild_id,
            Self::Autocomplete(x) => x.guild_id,
        }
    }

    /// Returns the channel ID of the underlying interaction
    pub fn channel_id(self) -> serenity::ChannelId {
        match self {
            Self::Command(x) => x.channel_id,
            Self::Autocomplete(x) => x.channel_id,
        }
    }

    /// Returns the member field of the underlying interaction
    pub fn member(self) -> Option<&'a serenity::Member> {
        match self {
            Self::Command(x) => x.member.as_deref(),
            Self::Autocomplete(x) => x.member.as_deref(),
        }
    }

    /// Returns the user field of the underlying interaction
    pub fn user(self) -> &'a serenity::User {
        match self {
            Self::Command(x) => &x.user,
            Self::Autocomplete(x) => &x.user,
        }
    }

    /// Returns the inner [`serenity::CommandInteraction`] and panics otherwise
    pub fn unwrap(self) -> &'a serenity::CommandInteraction {
        match self {
            CommandOrAutocompleteInteraction::Command(x) => x,
            CommandOrAutocompleteInteraction::Autocomplete(_) => {
                panic!("expected command interaction, got autocomplete interaction")
            }
        }
    }

    /// Returns the locale field of the underlying interaction
    pub fn locale(self) -> &'a str {
        match self {
            CommandOrAutocompleteInteraction::Command(x) => &x.locale,
            CommandOrAutocompleteInteraction::Autocomplete(x) => &x.locale,
        }
    }
}

/// Application command specific context passed to command invocations.
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub struct ApplicationContext<'a, U, E> {
    /// Serenity's context, like HTTP or cache
    #[derivative(Debug = "ignore")]
    pub discord: &'a serenity::Context,
    /// The interaction which triggered this command execution.
    pub interaction: CommandOrAutocompleteInteraction<'a>,
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
            CommandOrAutocompleteInteraction::Command(x) => x,
            CommandOrAutocompleteInteraction::Autocomplete(_) => return Ok(()),
        };

        if !self
            .has_sent_initial_response
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            interaction
                .create_response(
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

/// A single drop-down choice in a slash command choice parameter
#[derive(Debug, Clone)]
pub struct CommandParameterChoice {
    /// Label of this choice
    pub name: String,
    /// Localized labels with locale string as the key (slash-only)
    pub localizations: std::collections::HashMap<String, String>,
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
