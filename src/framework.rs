//! The central Framework struct that ties everything together.

use crate::serenity_prelude as serenity;
use crate::*;

// Adapted from serenity::Typing
#[derive(Debug)]
struct DelayedTyping(tokio::sync::oneshot::Sender<()>);
impl DelayedTyping {
    pub fn start(
        http: &std::sync::Arc<serenity::Http>,
        channel_id: serenity::ChannelId,
        delay: std::time::Duration,
    ) -> Self {
        let (sx, mut rx) = tokio::sync::oneshot::channel();

        let http = std::sync::Arc::clone(http);
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            loop {
                match rx.try_recv() {
                    Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => break,
                    _ => (),
                }

                channel_id.broadcast_typing(&http).await?;

                // It is unclear for how long typing persists after this method is called.
                // It is generally assumed to be 7 or 10 seconds, so we use 7 to be safe.
                tokio::time::sleep(std::time::Duration::from_secs(7)).await;
            }

            Ok::<_, serenity::Error>(())
        });

        Self(sx)
    }
}

pub struct Framework<U, E> {
    prefix: String,
    user_data: once_cell::sync::OnceCell<U>,
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
    // The bot ID is embedded in the token so we shouldn't have to do all of this mutex mess
    // But it's kinda messy to get access to the token in the framework
    bot_id: std::sync::Mutex<Option<serenity::UserId>>,
    // TODO: wrap in RwLock to allow changing framework options while running? Could also replace
    // the edit tracking cache interior mutability
    options: FrameworkOptions<U, E>,
    application_id: serenity::ApplicationId,
}

impl<U, E> Framework<U, E> {
    /// Setup a new blank Framework with a prefix and a callback to provide user data.
    ///
    /// The user data callback is invoked as soon as the bot is logged. That way, bot data like user
    /// ID or connected guilds can be made available to the user data setup function. The user data
    /// setup is not allowed to return Result because there would be no reasonable
    /// course of action on error.
    pub fn new<F>(
        prefix: String,
        application_id: serenity::ApplicationId,
        user_data_setup: F,
        options: FrameworkOptions<U, E>,
    ) -> Self
    where
        F: Send
            + Sync
            + 'static
            + for<'a> FnOnce(
                &'a serenity::Context,
                &'a serenity::Ready,
                &'a Self,
            ) -> BoxFuture<'a, Result<U, E>>,
    {
        Self {
            prefix,
            user_data: once_cell::sync::OnceCell::new(),
            user_data_setup: std::sync::Mutex::new(Some(Box::new(user_data_setup))),
            bot_id: std::sync::Mutex::new(None),
            options,
            application_id,
        }
    }

    pub async fn start(self, builder: serenity::ClientBuilder<'_>) -> Result<(), serenity::Error>
    where
        U: Send + Sync + 'static,
        E: 'static + Send,
    {
        let application_id = self.application_id;

        let self_1 = std::sync::Arc::new(self);
        let self_2 = std::sync::Arc::clone(&self_1);

        let edit_track_cache_purge_task = tokio::spawn(async move {
            loop {
                if let Some(edit_tracker) = &self_1.options.prefix_options.edit_tracker {
                    edit_tracker.write().purge();
                }
                // not sure if the purging interval should be configurable
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        let event_handler = EventWrapper(move |ctx, event| {
            let self_2 = std::sync::Arc::clone(&self_2);
            Box::pin(async move {
                self_2.event(ctx, event).await;
            }) as _
        });
        builder
            .application_id(application_id.0)
            .event_handler(event_handler)
            .await?
            .start()
            .await?;

        edit_track_cache_purge_task.abort();

        Ok(())
    }

    pub fn options(&self) -> &FrameworkOptions<U, E> {
        &self.options
    }

    pub fn application_id(&self) -> serenity::ApplicationId {
        self.application_id
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    // Commented out because it feels to specialized, and users will want to insert extra
    // bookkeeping anyways (e.g. number of slash commands, slash command names added, etc)

    // pub async fn register_slash_commands_in_guild(
    //     &self,
    //     http: &serenity::Http,
    //     guild_id: serenity::GuildId,
    // ) -> Result<(), serenity::Error> {
    //     for slash_cmd in &self.options.slash_options.commands {
    //         slash_cmd.create_in_guild(http, guild_id).await?;
    //     }
    //     Ok(())
    // }

    // pub async fn register_slash_commands_global(
    //     &self,
    //     http: &serenity::Http,
    // ) -> Result<(), serenity::Error> {
    //     for slash_cmd in &self.options.slash_options.commands {
    //         slash_cmd.create_global(http).await?;
    //     }
    //     Ok(())
    // }

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

    // Returns message with (only) bot prefix removed, if it matches
    async fn strip_prefix<'a>(
        &'a self,
        ctx: &'a serenity::Context,
        msg: &'a serenity::Message,
    ) -> Option<&'a str> {
        if let Some(content) = msg.content.strip_prefix(&self.prefix) {
            return Some(content);
        }

        if let Some(content) = self
            .options
            .prefix_options
            .additional_prefixes
            .iter()
            .find_map(|prefix| msg.content.strip_prefix(prefix))
        {
            return Some(content);
        }

        if self.options.prefix_options.mention_as_prefix {
            if let Some(content) = self.bot_id.lock().unwrap().and_then(|bot_id| {
                // Mentions are either <@USER_ID> or <@!USER_ID>
                msg.content
                    .strip_prefix("<@")?
                    .trim_start_matches('!')
                    .strip_prefix(&bot_id.0.to_string())?
                    .strip_prefix('>')
            }) {
                return Some(content);
            }
        }

        if let Some(dynamic_prefix) = self.options.prefix_options.dynamic_prefix {
            if let Some(content) = dynamic_prefix(ctx, msg, self.get_user_data().await).await {
                return Some(content);
            }
        }

        None
    }

    /// Find a command within nested PrefixCommandMeta's by the user message string. Also returns
    /// the arguments, i.e. the remaining string.
    ///
    /// May throw an error if a command check fails
    fn find_command<'a>(
        &'a self,
        ctx: &'a serenity::Context,
        msg: &'a serenity::Message,
        commands: &'a [crate::PrefixCommandMeta<U, E>],
        remaining_message: &'a str,
    ) -> crate::BoxFuture<
        'a,
        Result<
            Option<(&'a crate::PrefixCommandMeta<U, E>, &'a str)>,
            (E, PrefixCommandErrorContext<'a, U, E>),
        >,
    >
    where
        U: Send + Sync,
    {
        Box::pin(self._find_command(ctx, msg, commands, remaining_message))
    }

    async fn _find_command<'a>(
        &'a self,
        ctx: &'a serenity::Context,
        msg: &'a serenity::Message,
        commands: &'a [crate::PrefixCommandMeta<U, E>],
        remaining_message: &'a str,
    ) -> Result<
        Option<(&'a crate::PrefixCommandMeta<U, E>, &'a str)>,
        (E, PrefixCommandErrorContext<'a, U, E>),
    >
    where
        U: Send + Sync,
    {
        let considered_equal = if self.options.prefix_options.case_insensitive_commands {
            |a: &str, b: &str| a.eq_ignore_ascii_case(b)
        } else {
            |a: &str, b: &str| a == b
        };

        let (command_name, remaining_message) = {
            let mut iter = remaining_message.splitn(2, char::is_whitespace);
            (iter.next().unwrap(), iter.next().unwrap_or("").trim_start())
        };

        let mut first_matching_command = None;
        for command_meta in commands {
            let command = &command_meta.command;

            let primary_name_matches = considered_equal(command.name, command_name);
            let alias_matches = command
                .options
                .aliases
                .iter()
                .any(|alias| considered_equal(alias, command_name));
            if !primary_name_matches && !alias_matches {
                continue;
            }

            let prefix_ctx = prefix::PrefixContext {
                discord: ctx,
                msg,
                framework: self,
                data: self.get_user_data().await,
                command: Some(&command_meta.command),
            };

            match (command
                .options
                .check
                .unwrap_or(self.options.prefix_options.command_check))(prefix_ctx)
            .await
            {
                Ok(true) => {}
                Ok(false) => continue,
                Err(e) => {
                    return Err((
                        e,
                        prefix::PrefixCommandErrorContext {
                            command,
                            ctx: prefix_ctx,
                            while_checking: true,
                        },
                    ));
                }
            }

            first_matching_command = Some(
                if let Some((subcommand_meta, remaining_message)) = self
                    .find_command(ctx, msg, &command_meta.subcommands, remaining_message)
                    .await?
                {
                    (subcommand_meta, remaining_message)
                } else {
                    (command_meta, remaining_message)
                },
            );
            break;
        }

        Ok(first_matching_command)
    }

    /// Returns
    /// - Ok(()) if a command was successfully dispatched and run
    /// - Err(None) if the message does not match any known command
    /// - Err(Some(error: UserError)) if any user code yielded an error
    async fn dispatch_message<'a>(
        &'a self,
        ctx: &'a serenity::Context,
        msg: &'a serenity::Message,
        triggered_by_edit: bool,
    ) -> Result<(), Option<(E, PrefixCommandErrorContext<'a, U, E>)>>
    where
        U: Send + Sync,
    {
        // Strip prefix and whitespace between prefix and command
        let msg_content = self.strip_prefix(ctx, msg).await.ok_or(None)?.trim_start();

        // If we know our own ID, and the message author ID is our own, and we aren't supposed to
        // execute our own messages, THEN stop execution.
        if !self.options.prefix_options.execute_self_messages
            && *self.bot_id.lock().unwrap() == Some(msg.author.id)
        {
            return Err(None);
        }

        let (command_meta, args) = &self
            .find_command(ctx, msg, &self.options.prefix_options.commands, msg_content)
            .await
            .map_err(Some)?
            .ok_or(None)?;
        let command = &command_meta.command;

        if triggered_by_edit && !command.options.track_edits {
            return Err(None);
        }

        // Typing is broadcasted as long as this object is alive
        let _typing_broadcaster = match command
            .options
            .broadcast_typing
            .as_ref()
            .unwrap_or(&self.options.prefix_options.broadcast_typing)
        {
            BroadcastTypingBehavior::None => None,
            BroadcastTypingBehavior::WithDelay(delay) => {
                Some(DelayedTyping::start(&ctx.http, msg.channel_id, *delay))
            }
        };

        let ctx = prefix::PrefixContext {
            discord: &ctx,
            msg,
            framework: self,
            data: self.get_user_data().await,
            command: Some(command),
        };

        (self.options.pre_command)(Context::Prefix(ctx)).await;

        // Execute command
        (command.action)(ctx, args).await.map_err(|e| {
            Some((
                e,
                prefix::PrefixCommandErrorContext {
                    ctx,
                    command,
                    while_checking: false,
                },
            ))
        })
    }

    async fn dispatch_interaction(
        &self,
        ctx: &serenity::Context,
        interaction: &serenity::ApplicationCommandInteraction,
        name: &str,
        options: &[serenity::ApplicationCommandInteractionDataOption],
    ) {
        let command = match self
            .options
            .slash_options
            .commands
            .iter()
            .find(|cmd| cmd.name == name)
        {
            Some(x) => x,
            None => {
                println!("Warning: received unknown interaction \"{}\"", name);
                return;
            }
        };

        let has_sent_initial_response = std::sync::atomic::AtomicBool::new(false);
        let ctx = SlashContext {
            data: self.get_user_data().await,
            discord: &ctx,
            framework: self,
            interaction,
            command: &command,
            has_sent_initial_response: &has_sent_initial_response,
        };

        if command
            .options
            .defer_response
            .unwrap_or(self.options.slash_options.defer_response)
        {
            if let Err(e) = ctx.defer_response().await {
                println!("Failed to send interaction acknowledgement: {}", e);
            }
        }

        (self.options.pre_command)(Context::Slash(ctx)).await;

        if let Err(e) = (command.action)(ctx, options).await {
            let error_ctx = SlashCommandErrorContext {
                command,
                ctx,
                while_checking: false,
            };
            if let Some(on_error) = command.options.on_error {
                on_error(e, error_ctx).await;
            } else {
                (self.options.on_error)(
                    e,
                    ErrorContext::Command(CommandErrorContext::Slash(error_ctx)),
                )
                .await;
            }
        }
    }

    async fn event(&self, ctx: serenity::Context, event: Event<'_>)
    where
        U: Send + Sync,
    {
        match &event {
            Event::Ready { data_about_bot } => {
                *self.bot_id.lock().unwrap() = Some(data_about_bot.user.id);

                let user_data_setup = self.user_data_setup.lock().unwrap().take();
                if let Some(user_data_setup) = user_data_setup {
                    match user_data_setup(&ctx, &data_about_bot, self).await {
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
                    self.dispatch_message(&ctx, &new_message, false).await
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
                    let msg = edit_tracker.write().process_message_update(event);

                    if let Err(Some((err, ctx))) = self.dispatch_message(&ctx, &msg, true).await {
                        (self.options.on_error)(
                            err,
                            crate::ErrorContext::Command(crate::CommandErrorContext::Prefix(ctx)),
                        )
                        .await;
                    }
                }
            }
            Event::MessageDelete {
                deleted_message_id, ..
            } => {
                if let Some(edit_tracker) = &self.options.prefix_options.edit_tracker {
                    let bot_response = edit_tracker
                        .write()
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
                self.dispatch_interaction(
                    &ctx,
                    &interaction,
                    &interaction.data.name,
                    &interaction.data.options,
                )
                .await;
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
