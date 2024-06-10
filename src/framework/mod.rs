//! The central Framework struct that ties everything together.

use std::sync::Arc;

pub use builder::*;

use crate::serenity_prelude::{self as serenity, TeamMemberRole};

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
    /// Stores bot ID. Is initialized on first Ready event
    bot_id: std::sync::OnceLock<serenity::UserId>,
    /// Stores the framework options
    options: crate::FrameworkOptions<U, E>,

    /// Initialized to Some during construction; so shouldn't be None at any observable point
    shard_manager: Option<Arc<serenity::ShardManager>>,

    /// Handle to the background task in order to `abort()` it on `Drop`
    edit_tracker_purge_task: Option<tokio::task::JoinHandle<()>>,

    /// If [`Framework::dispatch`] should be called
    /// automatically (`true`) or manually by the developer (`false`)
    ///
    /// This allows the developer to make checks and call other functions before commands are
    /// handled at all.
    dispatch_automatically: bool,
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

    /// Setup a new [`Framework`].
    pub fn new(options: crate::FrameworkOptions<U, E>, dispatch_automatically: bool) -> Self
    where
        U: Send + Sync + 'static + 'static,
        E: Send + 'static,
    {
        Self {
            bot_id: std::sync::OnceLock::new(),
            edit_tracker_purge_task: None,
            shard_manager: None,
            options,
            dispatch_automatically,
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
}

impl<U, E> Drop for Framework<U, E> {
    fn drop(&mut self) {
        if let Some(task) = &mut self.edit_tracker_purge_task {
            task.abort()
        }
    }
}

#[serenity::async_trait]
impl<U: Send + Sync + 'static, E: Send + Sync> serenity::Framework for Framework<U, E> {
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

    async fn dispatch(&self, ctx: &serenity::Context, event: &serenity::FullEvent) {
        raw_dispatch_event(self, ctx, event).await
    }

    /// If [`Framework::dispatch`] should be called
    /// automatically (`true`) or manually by the developer (`false`)
    ///
    /// This allows the developer to make checks and call other functions before commands are
    /// handled at all.
    fn dispatch_automatically(&self) -> bool {
        self.dispatch_automatically
    }
}

/// If the incoming event is Ready, this method sets up [`Framework::bot_id`].
/// Otherwise, it forwards the event to [`crate::dispatch_event`]
async fn raw_dispatch_event<U, E>(
    framework: &Framework<U, E>,
    serenity_context: &serenity::Context,
    event: &serenity::FullEvent,
) where
    U: Send + Sync + 'static,
{
    if let serenity::FullEvent::Ready { data_about_bot } = event {
        let _: Result<_, _> = framework.bot_id.set(data_about_bot.user.id);
    }

    #[cfg(not(feature = "cache"))]
    let bot_id = *framework
        .bot_id
        .get()
        .expect("bot ID not set even though we awaited Ready");
    let framework = crate::FrameworkContext {
        #[cfg(not(feature = "cache"))]
        bot_id,
        serenity_context,
        options: &framework.options,
        shard_manager: framework.shard_manager(),
    };
    crate::dispatch_event(framework, event).await;
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
            // "Owner" is considered anyone with permission to access the token.
            if member.membership_state != serenity::MembershipState::Accepted {
                continue;
            }

            if let TeamMemberRole::Admin | TeamMemberRole::Developer = member.role {
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
