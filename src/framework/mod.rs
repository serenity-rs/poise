//! The central Framework struct that ties everything together.

// Prefix and slash specific implementation details
mod prefix;
mod slash;

mod builder;

pub use builder::*;

use crate::serenity::client::{bridge::gateway::ShardManager, Client};
use crate::serenity_prelude as serenity;
use crate::*;

pub use prefix::dispatch_message;

async fn check_permissions<U, E>(
    ctx: crate::Context<'_, U, E>,
    required_permissions: serenity::Permissions,
) -> bool {
    if required_permissions.is_empty() {
        return true;
    }

    let guild_id = match ctx.guild_id() {
        Some(x) => x,
        None => return true, // no permission checks in DMs
    };

    let guild = match ctx.discord().cache.guild(guild_id) {
        Some(x) => x,
        None => return false, // Guild not in cache
    };

    let channel = match guild.channels.get(&ctx.channel_id()) {
        Some(serenity::Channel::Guild(channel)) => channel,
        Some(_other_channel) => {
            println!(
                "Warning: guild message was supposedly sent in a non-guild channel. Denying invocation"
            );
            return false;
        }
        None => return false,
    };

    // If member not in cache (probably because presences intent is not enabled), retrieve via HTTP
    let member = match guild.members.get(&ctx.author().id) {
        Some(x) => x.clone(),
        None => match ctx
            .discord()
            .http
            .get_member(guild_id.0, ctx.author().id.0)
            .await
        {
            Ok(member) => member,
            Err(_) => return false,
        },
    };

    match guild.user_permissions_in(channel, &member) {
        Ok(perms) => perms.contains(required_permissions),
        Err(_) => false,
    }
}

async fn check_required_permissions_and_owners_only<U, E>(
    ctx: crate::Context<'_, U, E>,
    required_permissions: serenity::Permissions,
    owners_only: bool,
) -> bool {
    if owners_only && !ctx.framework().options().owners.contains(&ctx.author().id) {
        return false;
    }

    if !check_permissions(ctx, required_permissions).await {
        return false;
    }

    true
}

/// The main framework struct which stores all data and handles message and interaction dispatch.
pub struct Framework<U, E> {
    user_data: once_cell::sync::OnceCell<U>,
    bot_id: serenity::UserId,
    // TODO: wrap in RwLock to allow changing framework options while running? Could also replace
    // the edit tracking cache interior mutability
    options: FrameworkOptions<U, E>,
    application_id: serenity::ApplicationId,

    // Will be initialized to Some on construction, and then taken out on startup
    client: std::sync::Mutex<Option<serenity::Client>>,
    // Initialized to Some during construction; so shouldn't be None at any observable point
    shard_manager: std::sync::Mutex<Option<std::sync::Arc<tokio::sync::Mutex<ShardManager>>>>,
    // Filled with Some on construction. Taken out and executed on first Ready gateway event
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
        application_id: serenity::ApplicationId,
        client_builder: serenity::ClientBuilder<'_>,
        user_data_setup: F,
        options: FrameworkOptions<U, E>,
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
        let self_1 = std::sync::Arc::new(Self {
            user_data: once_cell::sync::OnceCell::new(),
            user_data_setup: std::sync::Mutex::new(Some(Box::new(user_data_setup))),
            bot_id: serenity::parse_token(client_builder.get_token().trim_start_matches("Bot "))
                .expect("Invalid bot token")
                .bot_user_id,
            // To break up the circular dependency (framework setup -> client setup -> event handler
            // -> framework), we initialize this with None and then immediately fill in once the
            // client is created
            client: std::sync::Mutex::new(None),
            options,
            application_id,
            shard_manager: std::sync::Mutex::new(None),
        });
        let self_2 = self_1.clone();

        let event_handler = EventWrapper(move |ctx, event| {
            let self_2 = std::sync::Arc::clone(&self_2);
            Box::pin(async move {
                self_2.event(ctx, event).await;
            }) as _
        });

        let client: Client = client_builder
            .application_id(application_id.0)
            .event_handler(event_handler)
            .await?;

        *self_1.shard_manager.lock().unwrap() = Some(client.shard_manager.clone());
        *self_1.client.lock().unwrap() = Some(client);

        Ok(self_1)
    }

    /// Start the framework.
    ///
    /// Takes a `serenity::ClientBuilder`, in which you need to supply the bot token, as well as
    /// any gateway intents.
    pub async fn start(self: std::sync::Arc<Self>) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: Send + 'static,
    {
        let mut client = self
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
        client.start().await?;

        edit_track_cache_purge_task.abort();

        Ok(())
    }

    /// Return the stored framework options, including commands.
    pub fn options(&self) -> &FrameworkOptions<U, E> {
        &self.options
    }

    /// Returns the application ID given to the framework on its creation.
    pub fn application_id(&self) -> serenity::ApplicationId {
        self.application_id
    }

    /// Returns the serenity's client shard manager.
    pub fn shard_manager(&self) -> std::sync::Arc<tokio::sync::Mutex<ShardManager>> {
        self.shard_manager
            .lock()
            .unwrap()
            .clone()
            .expect("fatal: shard manager not stored in framework initialization")
    }

    pub fn commands(&self) -> impl Iterator<Item = crate::CommandDefinitionRef<'_, U, E>> {
        type CommandMap<'s, U, E> =
            std::collections::HashMap<*const (), crate::CommandDefinitionRef<'s, U, E>>;

        fn get_command<'a, 's, U, E>(
            map: &'a mut CommandMap<'s, U, E>,
            id: &std::sync::Arc<crate::CommandId>,
        ) -> &'a mut crate::CommandDefinitionRef<'s, U, E> {
            map.entry(std::sync::Arc::as_ptr(id) as _)
                .or_insert_with(|| crate::CommandDefinitionRef {
                    prefix: None,
                    slash: None,
                    context_menu: None,
                    id: id.clone(),
                })
        }

        fn store_slash_commands<'a, U, E>(
            map: &mut CommandMap<'a, U, E>,
            command: &'a crate::SlashCommandMeta<U, E>,
        ) {
            match command {
                SlashCommandMeta::Command(command) => {
                    get_command(map, &command.id).slash = Some(command)
                }
                SlashCommandMeta::CommandGroup { subcommands, .. } => {
                    for subcommand in subcommands {
                        store_slash_commands(map, subcommand);
                    }
                }
            }
        }

        let mut map = CommandMap::new();
        for command in &self.options().application_options.commands {
            match command {
                ApplicationCommandTree::Slash(command) => store_slash_commands(&mut map, command),
                ApplicationCommandTree::ContextMenu(command) => {
                    get_command(&mut map, &command.id).context_menu = Some(command)
                }
            }
        }
        for command in &self.options().prefix_options.commands {
            get_command(&mut map, &command.command.id).prefix = Some(&command.command);
        }

        map.into_iter().map(|(_k, v)| v)
    }

    async fn get_user_data(&self) -> &U {
        // We shouldn't get a Message event before a Ready event. But if we do, wait until
        // the Ready event does come and the resulting data has arrived.
        loop {
            match self.user_data.get() {
                Some(x) => break x,
                None => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
            }
        }
    }

    async fn event(&self, ctx: serenity::Context, event: Event<'_>)
    where
        U: Send + Sync,
    {
        match &event {
            Event::Ready { data_about_bot } => {
                let user_data_setup = Option::take(&mut *self.user_data_setup.lock().unwrap());
                if let Some(user_data_setup) = user_data_setup {
                    match user_data_setup(&ctx, data_about_bot, self).await {
                        Ok(user_data) => {
                            let _: Result<_, _> = self.user_data.set(user_data);
                        }
                        Err(e) => (self.options.on_error)(e, ErrorContext::Setup).await,
                    }
                } else {
                    // discarding duplicate Discord bot ready event
                    // (happens regularly when bot is online for long period of time)
                }
            }
            Event::Message { new_message } => {
                if let Err(Some((err, ctx))) =
                    prefix::dispatch_message(self, &ctx, new_message, false).await
                {
                    if let Some(on_error) = ctx.command.options.on_error {
                        (on_error)(err, ctx).await;
                    } else {
                        (self.options.on_error)(
                            err,
                            crate::ErrorContext::Command(crate::CommandErrorContext::Prefix(ctx)),
                        )
                        .await;
                    }
                }
            }
            Event::MessageUpdate { event, .. } => {
                if let Some(edit_tracker) = &self.options.prefix_options.edit_tracker {
                    let msg = edit_tracker.write().unwrap().process_message_update(
                        event,
                        self.options().prefix_options.ignore_edit_tracker_cache,
                    );

                    if let Some(msg) = msg {
                        if let Err(Some((err, ctx))) =
                            prefix::dispatch_message(self, &ctx, &msg, true).await
                        {
                            (self.options.on_error)(
                                err,
                                crate::ErrorContext::Command(crate::CommandErrorContext::Prefix(
                                    ctx,
                                )),
                            )
                            .await;
                        }
                    }
                }
            }
            Event::MessageDelete {
                deleted_message_id, ..
            } => {
                if let Some(edit_tracker) = &self.options.prefix_options.edit_tracker {
                    let bot_response = edit_tracker
                        .write()
                        .unwrap()
                        .find_bot_response(*deleted_message_id)
                        .cloned();
                    if let Some(bot_response) = bot_response {
                        if let Err(e) = bot_response.delete(&ctx).await {
                            println!(
                                "Warning: couldn't delete bot response when user deleted message: {}",
                                e
                            );
                        }
                    }
                }
            }
            Event::InteractionCreate {
                interaction: serenity::Interaction::ApplicationCommand(interaction),
            } => {
                if let Err(Some((e, error_ctx))) = slash::dispatch_interaction(
                    self,
                    &ctx,
                    interaction,
                    &std::sync::atomic::AtomicBool::new(false),
                )
                .await
                {
                    if let Some(on_error) = error_ctx.ctx.command.options().on_error {
                        on_error(e, error_ctx).await;
                    } else {
                        (self.options.on_error)(
                            e,
                            ErrorContext::Command(CommandErrorContext::Application(error_ctx)),
                        )
                        .await;
                    }
                }
            }
            Event::InteractionCreate {
                interaction: serenity::Interaction::Autocomplete(interaction),
            } => {
                if let Err(Some((e, error_ctx))) = slash::dispatch_autocomplete(
                    self,
                    &ctx,
                    interaction,
                    &std::sync::atomic::AtomicBool::new(false),
                )
                .await
                {
                    if let Some(on_error) = error_ctx.ctx.command.options().on_error {
                        on_error(e, error_ctx).await;
                    } else {
                        (self.options.on_error)(e, ErrorContext::Autocomplete(error_ctx)).await;
                    }
                }
            }
            _ => {}
        }

        // Do this after the framework's Ready handling, so that self.get_user_data() doesnt
        // potentially block infinitely
        if let Err(e) =
            (self.options.listener)(&ctx, &event, self, self.get_user_data().await).await
        {
            (self.options.on_error)(e, ErrorContext::Listener(&event));
        }
    }
}
