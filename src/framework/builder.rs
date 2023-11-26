//! A builder struct that allows easy and readable creation of a [`crate::Framework`]

use crate::serenity_prelude as serenity;
use crate::BoxFuture;

/// A builder to configure a framework.
///
/// If one of the following required values is missing, the builder will panic on start:
/// - [`Self::setup`]
/// - [`Self::options`]
pub struct FrameworkBuilder<U: Send + Sync + 'static, E> {
    /// Callback for startup code and user data creation
    setup: Option<
        Box<
            dyn Send
                + Sync
                + for<'a> FnOnce(
                    &'a serenity::Context<U>,
                    &'a serenity::Ready,
                    &'a crate::Framework<U, E>,
                ) -> BoxFuture<'a, Result<(), E>>,
        >,
    >,
    /// Framework options
    options: Option<crate::FrameworkOptions<U, E>>,
    /// List of framework commands
    commands: Vec<crate::Command<U, E>>,
    /// See [`Self::initialize_owners()`]
    initialize_owners: bool,
}

impl<U: Send + Sync + 'static, E> Default for FrameworkBuilder<U, E> {
    fn default() -> Self {
        Self {
            setup: Default::default(),
            options: Default::default(),
            commands: Default::default(),
            initialize_owners: true,
        }
    }
}

impl<U: Send + Sync + 'static, E> FrameworkBuilder<U, E> {
    /// Sets the setup callback which also returns the user data struct.
    #[must_use]
    pub fn setup<F>(mut self, setup: F) -> Self
    where
        F: Send
            + Sync
            + 'static
            + for<'a> FnOnce(
                &'a serenity::Context<U>,
                &'a serenity::Ready,
                &'a crate::Framework<U, E>,
            ) -> BoxFuture<'a, Result<(), E>>,
    {
        self.setup = Some(Box::new(setup) as _);
        self
    }

    /// Configure framework options
    #[must_use]
    pub fn options(mut self, options: crate::FrameworkOptions<U, E>) -> Self {
        self.options = Some(options);
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
    pub fn build(self) -> crate::Framework<U, E>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        let setup = self
            .setup
            .expect("No user data setup function was provided to the framework");
        let mut options = self.options.expect("No framework options provided");

        // Build framework options by concatenating user-set options with commands and owners
        options.commands.extend(self.commands);
        options.initialize_owners = self.initialize_owners;

        // Create framework with specified settings
        crate::Framework::new(options, setup)
    }
}
