use crate::{serenity_prelude as serenity, BoxFuture};

fn prepare_command_definition<U, E>(
    definition: crate::CommandDefinition<U, E>,
    meta_builder: impl FnOnce(&mut CommandBuilder<U, E>) -> &mut CommandBuilder<U, E>,
) -> CommandBuilder<U, E> {
    // Unpack command implementations
    let crate::CommandDefinition {
        prefix: mut prefix_command,
        slash: mut slash_command,
        context_menu: mut context_menu_command,
        id,
    } = definition;

    // Make sure every implementation points to the same CommandId (they may have different
    // IDs if each implemented comes from a different function, like rustbot's rustify)
    if let Some(prefix_command) = &mut prefix_command {
        prefix_command.id = id.clone();
    }
    if let Some(slash_command) = &mut slash_command {
        slash_command.id = id.clone();
    }
    if let Some(context_menu_command) = &mut context_menu_command {
        context_menu_command.id = id.clone();
    }

    // Wrap the commands in their meta structs
    let prefix_command = prefix_command.map(|prefix_command| crate::PrefixCommandMeta {
        command: prefix_command,
        subcommands: Vec::new(),
    });
    let slash_command = slash_command.map(crate::SlashCommandMeta::Command);

    // Run the command builder on the meta structs to fill in metadata
    let mut builder = CommandBuilder {
        prefix_command,
        slash_command,
        context_menu_command,
        id,
    };
    meta_builder(&mut builder);

    builder
}

/// Builder struct to add a command to the framework
pub struct CommandBuilder<U, E> {
    prefix_command: Option<crate::PrefixCommandMeta<U, E>>,
    slash_command: Option<crate::SlashCommandMeta<U, E>>,
    context_menu_command: Option<crate::ContextMenuCommand<U, E>>,
    id: std::sync::Arc<crate::CommandId<U, E>>,
}

impl<U, E> CommandBuilder<U, E> {
    /// **Deprecated**
    #[deprecated = "Please use `category = \"...\"` on the command attribute instead"]
    pub fn category(&mut self, _category: &'static str) -> &mut Self {
        panic!("Please use `category = \"...\"` on the command attribute instead")
    }

    /// Insert a subcommand
    pub fn subcommand(
        &mut self,
        definition: crate::CommandDefinition<U, E>,
        meta_builder: impl FnOnce(&mut Self) -> &mut Self,
    ) -> &mut Self {
        let builder = prepare_command_definition(definition, meta_builder);

        // Nested if's to compile on Rust 1.48
        if let Some(parent) = &mut self.prefix_command {
            if let Some(subcommand) = builder.prefix_command {
                parent.subcommands.push(subcommand);
            }
        }

        if let Some(parent) = &mut self.slash_command {
            if let Some(subcommand) = builder.slash_command {
                match parent {
                    crate::SlashCommandMeta::CommandGroup { subcommands, .. } => {
                        subcommands.push(subcommand);
                    }
                    crate::SlashCommandMeta::Command(cmd) => {
                        *parent = crate::SlashCommandMeta::CommandGroup {
                            name: cmd.name,
                            description: cmd.description,
                            subcommands: vec![subcommand],
                            id: self.id.clone(),
                        };
                    }
                }
            }
        }

        self
    }
}

/// Framework configuration
pub struct FrameworkOptions<U, E> {
    /// Provide a callback to be invoked when any user code yields an error.
    pub on_error: fn(crate::FrameworkError<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called before every command
    pub pre_command: fn(crate::Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called after every command, no matter if it succeeded or failed
    pub post_command: fn(crate::Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// If individual commands add their own check, both callbacks are run and must return true.
    pub command_check: Option<fn(crate::Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Default set of allowed mentions to use for all responses
    pub allowed_mentions: Option<serenity::CreateAllowedMentions>,
    /// Called on every Discord event. Can be used to react to non-command events, like messages
    /// deletions or guild updates.
    pub listener: for<'a> fn(
        &'a serenity::Context,
        &'a crate::Event<'a>,
        &'a crate::Framework<U, E>,
        &'a U,
    ) -> BoxFuture<'a, Result<(), E>>,
    /// Application command specific options.
    pub application_options: crate::ApplicationFrameworkOptions<U, E>,
    /// Prefix command specific options.
    pub prefix_options: crate::PrefixFrameworkOptions<U, E>,
    /// User IDs which are allowed to use owners_only commands
    pub owners: std::collections::HashSet<serenity::UserId>,
}

impl<U, E> FrameworkOptions<U, E> {
    /// Add a command definition, which can include a prefix implementation, slash implementation,
    /// and context menu implementation, to the framework.
    ///
    /// To define subcommands or other meta information, pass a closure that calls the command
    /// builder
    ///
    /// ```rust
    /// # mod misc {
    /// #     type Error = Box<dyn std::error::Error + Send + Sync>;
    /// #     #[poise::command(prefix_command)]
    /// #     pub async fn ping(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// # use poise::FrameworkOptions;
    /// let mut options = FrameworkOptions::default();
    /// options.command(misc::ping(), |f| f);
    /// ```
    pub fn command(
        &mut self,
        definition: crate::CommandDefinition<U, E>,
        meta_builder: impl FnOnce(&mut CommandBuilder<U, E>) -> &mut CommandBuilder<U, E>,
    ) {
        let builder = prepare_command_definition(definition, meta_builder);

        // Insert command implementations
        if let Some(prefix_command) = builder.prefix_command {
            self.prefix_options.commands.push(prefix_command);
        }
        if let Some(slash_command) = builder.slash_command {
            self.application_options
                .commands
                .push(crate::ApplicationCommandTree::Slash(slash_command));
        }
        if let Some(context_menu_command) = builder.context_menu_command {
            self.application_options
                .commands
                .push(crate::ApplicationCommandTree::ContextMenu(
                    context_menu_command,
                ));
        }
    }
}

impl<U: std::fmt::Debug, E: std::fmt::Debug> std::fmt::Debug for FrameworkOptions<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            on_error,
            pre_command,
            post_command,
            command_check,
            allowed_mentions,
            listener,
            application_options,
            prefix_options,
            owners,
        } = self;

        f.debug_struct("FrameworkOptions")
            .field("on_error", &(*on_error as *const ()))
            .field("pre_command", &(*pre_command as *const ()))
            .field("post_command", &(*post_command as *const ()))
            .field("command_check", &command_check.map(|f| f as *const ()))
            .field("allowed_mentions", allowed_mentions)
            .field("listener", &(*listener as *const ()))
            .field("application_options", application_options)
            .field("prefix_options", prefix_options)
            .field("owners", owners)
            .finish()
    }
}

impl<U, E> Default for FrameworkOptions<U, E>
where
    U: Send + Sync + std::fmt::Debug,
    E: std::fmt::Display + std::fmt::Debug + Send,
{
    fn default() -> Self {
        Self {
            on_error: |error| {
                Box::pin(async move {
                    if let Err(e) = crate::builtins::on_error(error).await {
                        println!("Error while handling error: {}", e);
                    }
                })
            },
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            pre_command: |_| Box::pin(async {}),
            post_command: |_| Box::pin(async {}),
            command_check: None,
            allowed_mentions: Some({
                let mut f = serenity::CreateAllowedMentions::default();
                // Only support direct user pings by default
                f.empty_parse().parse(serenity::ParseValue::Users);
                f
            }),
            application_options: Default::default(),
            prefix_options: Default::default(),
            owners: Default::default(),
        }
    }
}
