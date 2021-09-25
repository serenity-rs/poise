use crate::serenity_prelude as serenity;
use crate::BoxFuture;

/// A builder to configure and run a framework.
///
/// If one of the following required values is missing, the builder will panic on start:
/// - [`Self::token`]
/// - [`Self::prefix`]
/// - [`Self::user_data_setup`]
/// - [`Self::options`]
///
/// Before starting, the builder will make an HTTP request to retrieve the bot's application ID and
/// owner.
pub struct FrameworkBuilder<U, E> {
    prefix: Option<String>,
    user_data_setup: Option<
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
    options: Option<crate::FrameworkOptions<U, E>>,
    client_settings:
        Option<Box<dyn FnOnce(serenity::ClientBuilder<'_>) -> serenity::ClientBuilder<'_>>>,
    token: Option<String>,
    intents: Option<serenity::GatewayIntents>,
    commands: Vec<(
        crate::CommandDefinition<U, E>,
        Box<dyn FnOnce(&mut crate::CommandBuilder<U, E>) -> &mut crate::CommandBuilder<U, E>>,
    )>,
}

impl<U, E> Default for FrameworkBuilder<U, E> {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            user_data_setup: Default::default(),
            options: Default::default(),
            client_settings: Default::default(),
            token: Default::default(),
            intents: Default::default(),
            commands: Default::default(),
        }
    }
}

impl<U, E> FrameworkBuilder<U, E> {
    /// Set a prefix for commands
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set a callback to be invoked to create the user data instance
    pub fn user_data_setup<F>(mut self, user_data_setup: F) -> Self
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
        self.user_data_setup = Some(Box::new(user_data_setup) as _);
        self
    }

    /// Configure framework options
    pub fn options(mut self, options: crate::FrameworkOptions<U, E>) -> Self {
        self.options = Some(options);
        self
    }

    /// Configure serenity client settings, like gateway intents, by supplying a custom
    /// client builder
    ///
    /// Note: the builder's token will be overridden by the
    /// [`FrameworkBuilder`]; use [`FrameworkBuilder::token`] to supply a token.
    pub fn client_settings(
        mut self,
        f: impl FnOnce(serenity::ClientBuilder<'_>) -> serenity::ClientBuilder<'_> + 'static,
    ) -> Self {
        self.client_settings = Some(Box::new(f));
        self
    }

    /// The bot token
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Add a new command to the framework
    pub fn command(
        mut self,
        definition: crate::CommandDefinition<U, E>,
        meta_builder: impl FnOnce(&mut crate::CommandBuilder<U, E>) -> &mut crate::CommandBuilder<U, E>
            + 'static,
    ) -> Self {
        self.commands.push((definition, Box::new(meta_builder)));
        self
    }

    /// Start the framework with the specified configuration.
    ///
    /// For more information, see [`FrameworkBuilder`]
    pub async fn run(self) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: 'static + Send,
    {
        // Aggregate required values or panic if not provided
        let token = self.token.expect("No token was provided to the framework");
        let prefix = self
            .prefix
            .expect("No prefix was provided to the framework");
        let user_data_setup = self
            .user_data_setup
            .expect("No user data setup function was provided to the framework");
        let mut options = self.options.expect("No framework options provided");

        // Retrieve application info via HTTP
        let application_info = serenity::Http::new_with_token(&token)
            .get_current_application_info()
            .await?;

        // Build framework options by concatenating user-set options with commands and owner
        for (command, meta_builder) in self.commands {
            options.command(command, meta_builder);
        }
        options.owners.insert(application_info.owner.id);

        // Create framework with specified settings
        let framework = crate::Framework {
            prefix,
            user_data: once_cell::sync::OnceCell::new(),
            user_data_setup: std::sync::Mutex::new(Some(user_data_setup)),
            bot_id: serenity::parse_token(&token)
                .expect("Invalid bot token")
                .bot_user_id,
            options,
            application_id: serenity::ApplicationId(application_info.id.0),
        };

        // Create serenity client
        let mut client = serenity::ClientBuilder::new(token)
            .application_id(application_info.id.0)
            .intents(
                self.intents
                    .unwrap_or_else(serenity::GatewayIntents::non_privileged),
            );
        if let Some(client_settings) = self.client_settings {
            client = client_settings(client);
        }

        // Run the framework
        framework.start(client).await
    }
}
