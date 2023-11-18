//! A builder struct that allows easy and readable creation of a [`crate::Framework`]

use crate::serenity_prelude as serenity;
use crate::BoxFuture;

/// A builder to configure and run a framework.
///
/// If one of the following required values is missing, the builder will panic on start:
/// - [`Self::token`]
/// - [`Self::setup`]
/// - [`Self::options`]
/// - [`Self::intents`]
///
/// Before starting, the builder will make an HTTP request to retrieve the bot's application ID and
/// owner, if [`Self::initialize_owners`] is set (true by default).
pub struct FrameworkBuilder<U, E> {
    /// Callback for startup code and user data creation
    setup: Option<
        Box<
            dyn Send
                + Sync
                + for<'a> FnOnce(
                    &'a serenity::Context,
                    &'a serenity::Ready,
                    &'a crate::Framework<U, E>,
                ) -> BoxFuture<'a, Result<U, E>>,
        >,
    >,
    /// Framework options
    options: Option<crate::FrameworkOptions<U, E>>,
    /// Client settings that will be applied to the ClientBuilder before initializing the framework
    client_settings:
        Option<Box<dyn FnOnce(serenity::ClientBuilder) -> serenity::ClientBuilder + Send + Sync>>,
    /// Discord bot token
    token: Option<String>,
    /// Discord gateway intents
    intents: Option<serenity::GatewayIntents>,
    /// List of framework commands
    commands: Vec<crate::Command<U, E>>,
    /// See [`Self::initialize_owners()`]
    initialize_owners: bool,
}

impl<U, E> Default for FrameworkBuilder<U, E> {
    fn default() -> Self {
        Self {
            setup: Default::default(),
            options: Default::default(),
            client_settings: Default::default(),
            token: Default::default(),
            intents: Default::default(),
            commands: Default::default(),
            initialize_owners: true,
        }
    }
}

impl<U, E> FrameworkBuilder<U, E> {
    /// Set a prefix for commands
    #[deprecated = "Please set the prefix via FrameworkOptions::prefix_options::prefix"]
    #[must_use]
    pub fn prefix(self, _prefix: impl Into<String>) -> Self {
        panic!("Please set the prefix via FrameworkOptions::prefix_options::prefix");
    }

    /// Sets the setup callback which also returns the user data struct.
    #[must_use]
    pub fn setup<F>(mut self, setup: F) -> Self
    where
        F: Send
            + Sync
            + 'static
            + for<'a> FnOnce(
                &'a serenity::Context,
                &'a serenity::Ready,
                &'a crate::Framework<U, E>,
            ) -> BoxFuture<'a, Result<U, E>>,
    {
        self.setup = Some(Box::new(setup) as _);
        self
    }

    /// Sets the setup callback which also returns the user data struct.
    #[must_use]
    #[deprecated = "renamed to .setup()"]
    pub fn user_data_setup<F>(self, setup: F) -> Self
    where
        F: Send
            + Sync
            + 'static
            + for<'a> FnOnce(
                &'a serenity::Context,
                &'a serenity::Ready,
                &'a crate::Framework<U, E>,
            ) -> BoxFuture<'a, Result<U, E>>,
    {
        self.setup(setup)
    }

    /// Configure framework options
    #[must_use]
    pub fn options(mut self, options: crate::FrameworkOptions<U, E>) -> Self {
        self.options = Some(options);
        self
    }

    /// Configure serenity client settings, like gateway intents, by supplying a custom
    /// client builder
    ///
    /// Note: the builder's token will be overridden by the
    /// [`FrameworkBuilder`]; use [`FrameworkBuilder::token`] to supply a token.
    #[must_use]
    pub fn client_settings(
        mut self,
        f: impl FnOnce(serenity::ClientBuilder) -> serenity::ClientBuilder + Send + Sync + 'static,
    ) -> Self {
        self.client_settings = Some(Box::new(f));
        self
    }

    /// The bot token
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// The gateway intents
    #[must_use]
    pub fn intents(mut self, intents: serenity::GatewayIntents) -> Self {
        self.intents = Some(intents);
        self
    }

    /// Add a new command to the framework
    #[deprecated = "supply commands in FrameworkOptions directly with `commands: vec![...]`"]
    #[must_use]
    pub fn command(
        mut self,
        mut command: crate::Command<U, E>,
        meta_builder: impl FnOnce(&mut crate::Command<U, E>) -> &mut crate::Command<U, E> + 'static,
    ) -> Self {
        meta_builder(&mut command);
        self.commands.push(command);
        self
    }

    /// Add multiple new commands to the framework. Shorthand for calling [`Self::command`] multiple
    /// times with the builder left to defaults, i.e. no command category or subcommands
    ///
    /// ```rust
    /// # type Error = Box<dyn std::error::Error + Send + Sync>;
    /// # #[poise::command(prefix_command)]
    /// # async fn command1(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> { Ok(()) }
    /// # #[poise::command(prefix_command)]
    /// # async fn command2(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> { Ok(()) }
    ///
    /// # #[allow(deprecated)] // just for this example
    /// poise::Framework::builder()
    ///     // framework setup...
    ///     .commands([command1, command2])
    ///     // framework startup...
    /// # ;
    /// ```
    #[deprecated = "supply commands in FrameworkOptions directly with `commands: vec![...]`"]
    #[must_use]
    pub fn commands(
        mut self,
        commands: impl IntoIterator<Item = fn() -> crate::Command<U, E>> + 'static,
    ) -> Self {
        self.commands.extend(commands.into_iter().map(|c| c()));
        self
    }

    /// Whether to add this bot application's owner and team members to
    /// [`crate::FrameworkOptions::owners`] automatically
    ///
    /// `true` by default
    pub fn initialize_owners(mut self, initialize_owners: bool) -> Self {
        self.initialize_owners = initialize_owners;
        self
    }

    /// Build the framework with the specified configuration.
    ///
    /// For more information, see [`FrameworkBuilder`]
    pub async fn build(self) -> Result<std::sync::Arc<crate::Framework<U, E>>, serenity::Error>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        // Aggregate required values or panic if not provided
        // WHEN CHANGING THESE, UPDATE FrameworkBuilder docs!
        let token = self.token.expect("No token was provided to the framework");
        let intents = self.intents.expect(
            "

No gateway intents were provided to the framework via `FrameworkBuilder::intents()`. If you're \
unsure, use
`serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT`
and enable MESSAGE_CONTENT in your Discord bot dashboard

",
        );
        let setup = self
            .setup
            .expect("No user data setup function was provided to the framework");
        let mut options = self.options.expect("No framework options provided");

        // Build framework options by concatenating user-set options with commands and owners
        options.commands.extend(self.commands);
        if self.initialize_owners {
            if let Err(e) = super::insert_owners_from_http(&token, &mut options.owners).await {
                tracing::warn!("Failed to insert owners from HTTP: {}", e);
            }
        }

        // Create serenity client
        let mut client_builder = serenity::ClientBuilder::new(token, intents);
        if let Some(client_settings) = self.client_settings {
            client_builder = client_settings(client_builder);
        }

        // Create framework with specified settings
        crate::Framework::new(client_builder, setup, options).await
    }

    /// Start the framework with the specified configuration.
    ///
    /// [`FrameworkBuilder::run`] is just a shorthand that calls [`FrameworkBuilder::build`] and
    /// starts the returned framework
    pub async fn run(self) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        self.build().await?.start().await
    }

    /// Autosharded version of [`Self::run`]
    pub async fn run_autosharded(self) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        self.build().await?.start_autosharded().await
    }
}
