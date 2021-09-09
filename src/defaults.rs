use crate::serenity_prelude as serenity;

/// This file provides a bunch of utility functions like help menus or error handlers to use as a
/// starting point for the framework.

type BoxErrorSendSync = Box<dyn std::error::Error + Send + Sync>;

/// An error handler that prints the error into the console and also into the Discord chat.
/// If the user invoked the command wrong
/// (i.e. an [`crate::ArgumentParseError`]), the command help is displayed and the user is directed
/// to the help menu.
pub async fn on_error<D>(e: BoxErrorSendSync, ctx: crate::ErrorContext<'_, D, BoxErrorSendSync>) {
    println!("Encountered an error: {:?}", e);
    match ctx {
        crate::ErrorContext::Command(ctx) => {
            let user_error_msg = if let Some(crate::ArgumentParseError(e)) = e.downcast_ref() {
                // If we caught an argument parse error, give a helpful error message with the
                // command explanation if available

                let mut usage = "Please check the help menu for usage information".into();
                if let crate::CommandErrorContext::Prefix(ctx) = &ctx {
                    if let Some(multiline_help) = &ctx.command.options.multiline_help {
                        usage = multiline_help();
                    }
                }
                format!("**{}**\n{}", e, usage)
            } else {
                e.to_string()
            };
            if let Err(e) = crate::say_reply(ctx.ctx(), user_error_msg).await {
                println!("Error while user command error: {}", e);
            }
        }
        crate::ErrorContext::Listener(event) => {
            println!("Error in listener while processing {:?}: {}", event, e)
        }
        crate::ErrorContext::Setup => println!("Setup failed: {}", e),
    }
}

/// Whether to send the help message as ephemeral (if possible), or as a normal message
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum HelpResponseMode {
    /// Normal non-ephemeral message
    Default,
    /// Ephemeral message
    Ephemeral,
}

/// A help command that outputs text in a code block, groups commands by categories, and annotates
/// commands with a slash if they exist as slash commands.
///
/// Example usage from Ferris, the Discord bot running in the Rust community server:
/// ```rust
/// # type Error = Box<dyn std::error::Error>;
/// # type Context<'a> = poise::Context<'a, (), Error>;
/// /// Show this menu
/// #[poise::command(prefix_command, track_edits, slash_command)]
/// pub async fn help(
///     ctx: Context<'_>,
///     #[description = "Specific command to show help about"] command: Option<String>,
/// ) -> Result<(), Error> {
///     let bottom_text = "Type ?help command for more info on a command.
/// You can edit your message to the bot and the bot will edit its response.";
///     poise::defaults::help(
///         ctx,
///         command.as_deref(),
///         bottom_text,
///         poise::defaults::HelpResponseMode::Ephemeral,
///     )
///     .await?;
///     Ok(())
/// }
/// ```
/// Output:
/// ```text
/// Playground:
///   ?play        Compile and run Rust code in a playground
///   ?eval        Evaluate a single Rust expression
///   ?miri        Run code and detect undefined behavior using Miri
///   ?expand      Expand macros to their raw desugared form
///   ?clippy      Catch common mistakes using the Clippy linter
///   ?fmt         Format code using rustfmt
///   ?microbench  Benchmark small snippets of code
///   ?procmacro   Compile and use a procedural macro
///   ?godbolt     View assembly using Godbolt
///   ?mca         Run performance analysis using llvm-mca
///   ?llvmir      View LLVM IR using Godbolt
/// Crates:
///   /crate       Lookup crates on crates.io
///   /doc         Lookup documentation
/// Moderation:
///   /cleanup     Deletes the bot's messages for cleanup
///   /ban         Bans another person
///   ?move        Move a discussion to another channel
///   /rustify     Adds the Rustacean role to members
/// Miscellaneous:
///   ?go          Evaluates Go code
///   /source      Links to the bot GitHub repo
///   /help        Show this menu
///
/// Type ?help command for more info on a command.
/// You can edit your message to the bot and the bot will edit its response.
/// ```
pub async fn help<D, E>(
    ctx: crate::Context<'_, D, E>,
    command: Option<&str>,
    extra_text_at_bottom: &str,
    response_mode: HelpResponseMode,
) -> Result<(), serenity::Error> {
    let reply = if let Some(command) = command {
        if let Some(command) = ctx
            .framework()
            .options()
            .prefix_options
            .commands
            .iter()
            .map(|cmd_meta| &cmd_meta.command)
            .find(|cmd| cmd.name == command)
        {
            match command.options.multiline_help {
                Some(f) => f(),
                None => command
                    .options
                    .inline_help
                    .unwrap_or("No help available")
                    .to_owned(),
            }
        } else {
            format!("No such command `{}`", command)
        }
    } else {
        let is_also_a_slash_command = |command_name| {
            let application_commands = &ctx.framework().options().application_options.commands;
            application_commands.iter().any(|c| match c {
                crate::ApplicationCommandTree::Slash(cmd) => match cmd {
                    crate::SlashCommandMeta::Command(cmd) => cmd.name == command_name,
                    crate::SlashCommandMeta::CommandGroup { .. } => false, // STUB
                },
                _ => false,
            })
        };

        let mut categories: Vec<(Option<&str>, Vec<&crate::PrefixCommand<_, _>>)> = Vec::new();
        for cmd_meta in &ctx.framework().options().prefix_options.commands {
            if let Some((_, commands)) = categories
                .iter_mut()
                .find(|(key, _)| *key == cmd_meta.category)
            {
                commands.push(&cmd_meta.command);
            } else {
                categories.push((cmd_meta.category, vec![&cmd_meta.command]));
            }
        }

        let mut menu = String::from("```\n");
        for (category_name, commands) in categories {
            menu += category_name.unwrap_or("Commands");
            menu += ":\n";
            for command in commands {
                if command.options.hide_in_help {
                    continue;
                }

                let prefix = if is_also_a_slash_command(command.name) {
                    "/"
                } else {
                    ctx.framework().prefix()
                };

                menu += &format!(
                    "  {}{:<12}{}\n",
                    prefix,
                    command.name,
                    command.options.inline_help.unwrap_or("")
                );
            }
        }
        menu += "\n";
        menu += extra_text_at_bottom;
        menu += "\n```";

        menu
    };

    let ephemeral = match response_mode {
        HelpResponseMode::Default => false,
        HelpResponseMode::Ephemeral => true,
    };

    crate::send_reply(ctx, |f| f.content(reply).ephemeral(ephemeral)).await?;

    Ok(())
}

/// Generic function to register application commands. Only allows server owners to invoke.
///
/// If you want, you can copy paste this help message:
///
/// ```ignore
/// Register application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
/// ```
pub async fn register_application_commands<U, E>(
    ctx: crate::Context<'_, U, E>,
    global: bool,
) -> Result<(), serenity::Error> {
    let guild = match ctx.guild() {
        Some(x) => x,
        None => {
            crate::say_reply(ctx, "Must be called in guild").await?;
            return Ok(());
        }
    };

    if ctx.author().id != guild.owner_id {
        crate::say_reply(ctx, "Can only be used by server owner").await?;
        return Ok(());
    }

    let mut commands_builder = serenity::CreateApplicationCommands::default();
    let commands = &ctx.framework().options().application_options.commands;
    for cmd in commands {
        commands_builder.create_application_command(|f| cmd.create(f));
    }

    crate::say_reply(ctx, format!("Registering {} commands...", commands.len())).await?;
    let json_value = serde_json::Value::Array(commands_builder.0);
    if global {
        ctx.discord()
            .http
            .create_global_application_commands(&json_value)
            .await?;
    } else {
        ctx.discord()
            .http
            .create_guild_application_commands(guild.id.0, &json_value)
            .await?;
    }
    crate::say_reply(ctx, "Done!").await?;

    Ok(())
}
