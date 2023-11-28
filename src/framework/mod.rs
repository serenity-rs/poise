//! The central Framework struct that ties everything together.

use std::sync::Arc;

pub use builder::*;

use crate::{serenity_prelude as serenity, BoxFuture};

mod builder;

/// The main framework struct which stores all data and handles message and interaction dispatch.
///
/// Technically, this is just an optional abstraction over [`crate::dispatch_event`] with some
/// additional conveniences built-in:
/// - fills in correct values for [`crate::Command::qualified_name`]: [`set_qualified_names`]
/// - spawns a background task to periodically clear edit tracker cache
/// - sets up user data on the first Ready event
/// - keeps track of shard manager and bot ID automatically
///
/// You can build a bot without [`Framework`]: see the `manual_dispatch` example in the repository
pub struct Framework<U, E> {
    /// Stores user data. Is initialized on first Ready event
    user_data: std::sync::OnceLock<U>,
    /// Stores bot ID. Is initialized on first Ready event
    bot_id: std::sync::OnceLock<serenity::UserId>,
    /// Stores the framework options
    options: crate::FrameworkOptions<U, E>,

    /// Initialized to Some during construction; so shouldn't be None at any observable point
    shard_manager: Option<Arc<serenity::ShardManager>>,
    /// Filled with Some on construction. Taken out and executed on first Ready gateway event
    setup: std::sync::Mutex<
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

    /// Handle to the background task in order to `abort()` it on `Drop`
    edit_tracker_purge_task: Option<tokio::task::JoinHandle<()>>,
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
    /// The user data callback is invoked as soon as the bot is logged in. That way, bot data like
    /// user ID or connected guilds can be made available to the user data setup function. The user
    /// data setup is not allowed to return Result because there would be no reasonable
    /// course of action on error.
    pub fn new<F>(options: crate::FrameworkOptions<U, E>, setup: F) -> Self
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
        Self {
            user_data: std::sync::OnceLock::new(),
            bot_id: std::sync::OnceLock::new(),
            setup: std::sync::Mutex::new(Some(Box::new(setup))),
            edit_tracker_purge_task: None,
            shard_manager: None,
            options,
        }
    }

    /// Return the stored framework options, including commands.
    pub fn options(&self) -> &crate::FrameworkOptions<U, E> {
        &self.options
    }

    /// Returns the serenity's client shard manager.
    // Returns a reference so you can plug it into [`FrameworkContext`]
    pub fn shard_manager(&self) -> &Arc<serenity::ShardManager> {
        self.shard_manager
            .as_ref()
            .expect("framework should have started")
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

impl<U, E> Drop for Framework<U, E> {
    fn drop(&mut self) {
        if let Some(task) = &mut self.edit_tracker_purge_task {
            task.abort()
        }
    }
}

#[serenity::async_trait]
impl<U: Send + Sync, E: Send + Sync> serenity::Framework for Framework<U, E> {
    async fn init(&mut self, client: &serenity::Client) {
        set_qualified_names(&mut self.options.commands);

        message_content_intent_sanity_check(
            &self.options.prefix_options,
            client.shard_manager.intents(),
        );

        self.shard_manager = Some(client.shard_manager.clone());

        if self.options.initialize_owners {
            if let Err(e) = insert_owners_from_http(&client.http, &mut self.options.owners).await {
                tracing::warn!("Failed to insert owners from HTTP: {e}");
            }
        }

        if let Some(edit_tracker) = &self.options.prefix_options.edit_tracker {
            self.edit_tracker_purge_task =
                Some(spawn_edit_tracker_purge_task(edit_tracker.clone()));
        }
    }

    async fn dispatch(&self, ctx: serenity::Context, event: serenity::FullEvent) {
        raw_dispatch_event(self, ctx, event).await
    }
}

/// If the incoming event is Ready, this method executes the user data setup logic
/// Otherwise, it forwards the event to [`crate::dispatch_event`]
async fn raw_dispatch_event<U, E>(
    framework: &Framework<U, E>,
    ctx: serenity::Context,
    event: serenity::FullEvent,
) where
    U: Send + Sync,
{
    if let serenity::FullEvent::Ready { data_about_bot } = &event {
        let _: Result<_, _> = framework.bot_id.set(data_about_bot.user.id);
        let setup = Option::take(&mut *framework.setup.lock().unwrap());
        if let Some(setup) = setup {
            match setup(&ctx, data_about_bot, framework).await {
                Ok(user_data) => {
                    let _: Result<_, _> = framework.user_data.set(user_data);
                }
                Err(error) => {
                    (framework.options.on_error)(crate::FrameworkError::Setup {
                        error,
                        framework,
                        data_about_bot,
                        ctx: &ctx,
                    })
                    .await
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
        shard_manager: framework.shard_manager(),
    };
    crate::dispatch_event(framework, &ctx, event).await;
}

/// Traverses commands recursively and sets [`crate::Command::qualified_name`] to its actual value
pub fn set_qualified_names<U, E>(commands: &mut [crate::Command<U, E>]) {
    /// Fills in `qualified_name` fields by appending command name to the parent command name
    fn set_subcommand_qualified_names<U, E>(parents: &str, commands: &mut [crate::Command<U, E>]) {
        for cmd in commands {
            cmd.qualified_name = format!("{} {}", parents, cmd.name);
            set_subcommand_qualified_names(&cmd.qualified_name, &mut cmd.subcommands);
        }
    }
    for command in commands {
        set_subcommand_qualified_names(&command.name, &mut command.subcommands);
    }
}

/// Prints a warning on stderr if a prefix is configured but `MESSAGE_CONTENT` is not set
fn message_content_intent_sanity_check<U, E>(
    prefix_options: &crate::PrefixFrameworkOptions<U, E>,
    intents: serenity::GatewayIntents,
) {
    let is_prefix_configured = prefix_options.prefix.is_some()
        || prefix_options.dynamic_prefix.is_some()
        || prefix_options.stripped_dynamic_prefix.is_some();
    let can_receive_message_content = intents.contains(serenity::GatewayIntents::MESSAGE_CONTENT);
    if is_prefix_configured && !can_receive_message_content {
        tracing::warn!(
            "Warning: MESSAGE_CONTENT intent not set; prefix commands will not be received"
        );
    }
}

/// Runs [`serenity::Http::get_current_application_info`] and inserts owner data into
/// [`crate::FrameworkOptions::owners`]
pub async fn insert_owners_from_http(
    http: &serenity::Http,
    owners: &mut std::collections::HashSet<serenity::UserId>,
) -> Result<(), serenity::Error> {
    let application_info = http.get_current_application_info().await?;

    if let Some(owner) = application_info.owner {
        owners.insert(owner.id);
    }

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
/// Because e.g. taking a `PrefixFrameworkOptions` reference won't work because tokio tasks need to be
/// 'static
fn spawn_edit_tracker_purge_task(
    edit_tracker: Arc<std::sync::RwLock<crate::EditTracker>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            edit_tracker.write().unwrap().purge();

            // not sure if the purging interval should be configurable
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    })
}
