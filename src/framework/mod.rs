//! The central Framework struct that ties everything together.

mod dispatch;

mod builder;
pub use builder::*;

use crate::{serenity_prelude as serenity, BoxFuture};

pub use dispatch::{dispatch_message, find_command};

/// The main framework struct which stores all data and handles message and interaction dispatch.
pub struct Framework<U, E> {
    /// Stores user data. Is initialized on first Ready event
    user_data: once_cell::sync::OnceCell<U>,
    /// Stores the framework options
    options: crate::FrameworkOptions<U, E>,

    /// Will be initialized to Some on construction, and then taken out on startup
    client: std::sync::Mutex<Option<serenity::Client>>,
    /// Initialized to Some during construction; so shouldn't be None at any observable point
    shard_manager: std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>>,
    /// Filled with Some on construction. Taken out and executed on first Ready gateway event
    user_data_setup: std::sync::Mutex<
        Option<
            Box<
                dyn Send
                    + Sync
                    + for<'a> FnOnce(
                        &'a serenity::Context,
                        &'a serenity::Ready,
                        &'a Self,
                    ) -> BoxFuture<'a, Result<U, E>>,
            >,
        >,
    >,
}

impl<U, E> Framework<U, E> {
    /// Create a framework builder to configure, create and run a framework.
    ///
    /// For more information, see [`FrameworkBuilder`]
    pub fn build() -> FrameworkBuilder<U, E> {
        FrameworkBuilder::default()
    }

    /// Setup a new [`Framework`]. For more ergonomic setup, please see [`FrameworkBuilder`]
    ///
    /// This function is async and returns Result because it already initializes the Discord client.
    ///
    /// The user data callback is invoked as soon as the bot is logged in. That way, bot data like
    /// user ID or connected guilds can be made available to the user data setup function. The user
    /// data setup is not allowed to return Result because there would be no reasonable
    /// course of action on error.
    pub async fn new<F>(
        client_builder: serenity::ClientBuilder,
        user_data_setup: F,
        options: crate::FrameworkOptions<U, E>,
    ) -> Result<std::sync::Arc<Self>, serenity::Error>
    where
        F: Send
            + Sync
            + 'static
            + for<'a> FnOnce(
                &'a serenity::Context,
                &'a serenity::Ready,
                &'a Self,
            ) -> BoxFuture<'a, Result<U, E>>,
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        use std::sync::{Arc, Mutex};

        let framework_cell = Arc::new(once_cell::sync::OnceCell::<Arc<Self>>::new());
        let framework_cell_2 = framework_cell.clone();
        let event_handler = crate::EventWrapper(move |ctx, event| {
            // unwrap_used: we will only receive events once the client has been started, by which
            // point framework_cell has been initialized
            #[allow(clippy::unwrap_used)]
            let framework = framework_cell_2.get().unwrap().clone();
            Box::pin(async move { dispatch::dispatch_event(&*framework, ctx, &event).await }) as _
        });

        let client: serenity::Client = client_builder.event_handler(event_handler).await?;

        let framework = Arc::new(Self {
            user_data: once_cell::sync::OnceCell::new(),
            user_data_setup: Mutex::new(Some(Box::new(user_data_setup))),
            options,
            shard_manager: client.shard_manager.clone(),
            client: Mutex::new(Some(client)),
        });
        let _: Result<_, _> = framework_cell.set(framework.clone());
        Ok(framework)
    }

    /// Small utility function for starting the framework that is agnostic over client sharding
    async fn start_with<F: std::future::Future<Output = serenity::Result<()>>>(
        self: std::sync::Arc<Self>,
        start: fn(serenity::Client) -> F,
    ) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        let client = self
            .client
            .lock()
            .unwrap()
            .take()
            .expect("Prepared client is missing");

        let edit_track_cache_purge_task = tokio::spawn(async move {
            loop {
                if let Some(edit_tracker) = &self.options.prefix_options.edit_tracker {
                    edit_tracker.write().unwrap().purge();
                }
                // not sure if the purging interval should be configurable
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        // This will run for as long as the bot is active
        start(client).await?;

        edit_track_cache_purge_task.abort();

        Ok(())
    }

    /// Starts the framework.
    pub async fn start(self: std::sync::Arc<Self>) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        self.start_with(|mut c| async move { c.start().await })
            .await
    }

    /// Starts the framework. Calls [`serenity::Client::start_autosharded`] internally
    pub async fn start_autosharded(self: std::sync::Arc<Self>) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        self.start_with(|mut c| async move { c.start_autosharded().await })
            .await
    }

    /// Return the stored framework options, including commands.
    pub fn options(&self) -> &crate::FrameworkOptions<U, E> {
        &self.options
    }

    /// Returns the serenity's client shard manager.
    pub fn shard_manager(&self) -> std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>> {
        self.shard_manager.clone()
    }

    /// Retrieves user data, or blocks until it has been initialized (once the Ready event has been
    /// received).
    pub async fn user_data(&self) -> &U {
        loop {
            match self.user_data.get() {
                Some(x) => break x,
                None => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            }
        }
    }
}
