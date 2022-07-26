//! The central Framework struct that ties everything together.

mod builder;
pub use builder::*;

use crate::{serenity_prelude as serenity, BoxFuture};

/// The main framework struct which stores all data and handles message and interaction dispatch.
///
/// Technically, this is just an optional abstraction over [`crate::dispatch_event`] with some
/// additional conveniences built-in:
/// - fills in correct values for [`crate::Command::qualified_name`]: [`set_qualified_names`]
/// - spawns a background task to periodically clear edit tracker cache
/// - sets up user data on the first Ready event
/// - keeps track of shard manager and bot ID automatically
///
/// You can build a bot without [`Framework`]: see the manual_dispatch example in the repository
pub struct Framework<U, E> {
    /// Stores user data. Is initialized on first Ready event
    user_data: once_cell::sync::OnceCell<U>,
    /// Stores bot ID. Is initialized on first Ready event
    bot_id: once_cell::sync::OnceCell<serenity::UserId>,
    /// Stores the framework options
    options: crate::FrameworkOptions<U, E>,

    /// Will be initialized to Some on construction, and then taken out on startup
    client: parking_lot::Mutex<Option<serenity::Client>>,
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
    #[deprecated = "Please use Framework::builder instead"]
    pub fn build() -> FrameworkBuilder<U, E> {
        FrameworkBuilder::default()
    }

    /// Create a framework builder to configure, create and run a framework.
    ///
    /// For more information, see [`FrameworkBuilder`]
    pub fn builder() -> FrameworkBuilder<U, E> {
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
        mut options: crate::FrameworkOptions<U, E>,
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

        set_qualified_names(&mut options.commands);
        message_content_intent_sanity_check(&options.prefix_options, client_builder.get_intents());

        let framework_cell = Arc::new(once_cell::sync::OnceCell::<Arc<Self>>::new());
        let framework_cell_2 = framework_cell.clone();
        let existing_event_handler = client_builder.get_event_handler();
        let event_handler = crate::EventWrapper(move |ctx, event| {
            // unwrap_used: we will only receive events once the client has been started, by which
            // point framework_cell has been initialized
            #[allow(clippy::unwrap_used)]
            let framework = framework_cell_2.get().unwrap().clone();
            let existing_event_handler = existing_event_handler.clone();

            Box::pin(async move {
                raw_dispatch_event(&*framework, &ctx, &event).await;
                if let Some(handler) = existing_event_handler {
                    event.dispatch(ctx, &*handler).await;
                }
            }) as _
        });

        let client: serenity::Client = client_builder.event_handler(event_handler).await?;

        let framework = Arc::new(Self {
            user_data: once_cell::sync::OnceCell::new(),
            bot_id: once_cell::sync::OnceCell::new(),
            user_data_setup: Mutex::new(Some(Box::new(user_data_setup))),
            options,
            shard_manager: client.shard_manager.clone(),
            client: parking_lot::Mutex::new(Some(client)),
        });
        let _: Result<_, _> = framework_cell.set(framework.clone());
        Ok(framework)
    }

    /// Small utility function for starting the framework that is agnostic over client sharding
    ///
    /// Used internally by [`Self::start()`] and [`Self::start_autosharded()`]
    pub async fn start_with<F: std::future::Future<Output = serenity::Result<()>>>(
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
            .take()
            .expect("Prepared client is missing");

        // This will run for as long as the bot is active
        let edit_tracker_purge_task = spawn_edit_tracker_purge_task(self);
        start(client).await?;
        edit_tracker_purge_task.abort();

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
    // Returns a reference so you can plug it into [`FrameworkContext`]
    pub fn shard_manager(&self) -> &std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>> {
        &self.shard_manager
    }

    /// Returns the serenity client. Panics if the framework has already started!
    pub fn client(&self) -> impl std::ops::DerefMut<Target = serenity::Client> + '_ {
        parking_lot::MutexGuard::map(self.client.lock(), |c| {
            c.as_mut().expect("framework has started")
        })
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

/// If the incoming event is Ready, this method executes the user data setup logic
/// Otherwise, it forwards the event to [`crate::dispatch_event`]
async fn raw_dispatch_event<U, E>(
    framework: &crate::Framework<U, E>,
    ctx: &serenity::Context,
    event: &crate::Event<'_>,
) where
    U: Send + Sync,
{
    if let crate::Event::Ready { data_about_bot } = event {
        let _: Result<_, _> = framework.bot_id.set(data_about_bot.user.id);
        let user_data_setup = Option::take(&mut *framework.user_data_setup.lock().unwrap());
        if let Some(user_data_setup) = user_data_setup {
            match user_data_setup(ctx, data_about_bot, framework).await {
                Ok(user_data) => {
                    let _: Result<_, _> = framework.user_data.set(user_data);
                }
                Err(error) => {
                    (framework.options.on_error)(crate::FrameworkError::Setup { error }).await
                }
            }
        } else {
            // ignoring duplicate Discord bot ready event
            // (happens regularly when bot is online for long period of time)
        }
    }

    let user_data = framework.user_data().await;
    let bot_id = *framework
        .bot_id
        .get()
        .expect("bot ID not set even though we awaited Ready");
    let framework = crate::FrameworkContext {
        bot_id,
        options: &framework.options,
        user_data,
        shard_manager: &framework.shard_manager,
    };
    crate::dispatch_event(framework, ctx, event).await;
}

/// Traverses commands recursively and sets [`crate::Command::qualified_name`] to its actual value
pub fn set_qualified_names<U, E>(commands: &mut [crate::Command<U, E>]) {
    /// Fills in qualified_name fields by appending command name to the parent command name
    fn set_subcommand_qualified_names<U, E>(parents: &str, commands: &mut [crate::Command<U, E>]) {
        for cmd in commands {
            cmd.qualified_name = format!("{} {}", parents, cmd.name);
            set_subcommand_qualified_names(&cmd.qualified_name, &mut cmd.subcommands);
        }
    }
    for command in commands {
        set_subcommand_qualified_names(command.name, &mut command.subcommands);
    }
}

/// Prints a warning on stderr if a prefix is configured but MESSAGE_CONTENT is not set
fn message_content_intent_sanity_check<U, E>(
    prefix_options: &crate::PrefixFrameworkOptions<U, E>,
    intents: serenity::GatewayIntents,
) {
    let is_prefix_configured = prefix_options.prefix.is_some()
        || prefix_options.dynamic_prefix.is_some()
        || prefix_options.stripped_dynamic_prefix.is_some();
    let can_receive_message_content = intents.contains(serenity::GatewayIntents::MESSAGE_CONTENT);
    if is_prefix_configured && !can_receive_message_content {
        eprintln!("Warning: MESSAGE_CONTENT intent not set; prefix commands will not be received");
    }
}

/// Runs [`serenity::Http::get_current_application_info`] and inserts owner data into
/// [`crate::FrameworkOptions::owners`]
pub async fn insert_owners_from_http(
    token: &str,
    owners: &mut std::collections::HashSet<serenity::UserId>,
) -> Result<(), serenity::Error> {
    let application_info = serenity::Http::new(token)
        .get_current_application_info()
        .await?;

    owners.insert(application_info.owner.id);
    if let Some(team) = application_info.team {
        for member in team.members {
            // This `if` currently always evaluates to true but it becomes important once
            // Discord implements more team roles than Admin
            if member.permissions.iter().any(|p| p == "*") {
                owners.insert(member.user.id);
            }
        }
    }

    Ok(())
}

/// Spawns a background task that periodically purges outdated entries from the edit tracker cache
///
/// Important to avoid the edit tracker gobbling up unlimited memory
///
/// NOT PUB because it's not useful to outside users because it requires a full blown Framework
/// Because e.g. taking a PrefixFrameworkOptions reference won't work because tokio tasks need to be
/// 'static
fn spawn_edit_tracker_purge_task<U: 'static + Send + Sync, E: 'static>(
    framework: std::sync::Arc<Framework<U, E>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
            loop {
                edit_tracker.write().unwrap().purge();

                // not sure if the purging interval should be configurable
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        }
    })
}
