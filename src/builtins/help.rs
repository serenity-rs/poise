//! Contains the built-in help command and surrounding infrastructure

use crate::serenity_prelude as serenity;
use std::fmt::Write as _;

/// Optional configuration for how the help message from [`help()`] looks
pub struct HelpConfiguration<'a> {
    /// Extra text displayed at the bottom of your message. Can be used for help and tips specific
    /// to your bot
    pub extra_text_at_bottom: &'a str,
    /// Whether to make the response ephemeral if possible. Can be nice to reduce clutter
    pub ephemeral: bool,
    /// Whether to list context menu commands as well
    pub show_context_menu_commands: bool,
    /// Whether to list context menu commands as well
    pub show_subcommands: bool,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for HelpConfiguration<'_> {
    fn default() -> Self {
        Self {
            extra_text_at_bottom: "",
            ephemeral: true,
            show_context_menu_commands: false,
            show_subcommands: false,
            __non_exhaustive: (),
        }
    }
}

/// Code for printing help of a specific command (e.g. `~help my_command`)
async fn help_single_command<U, E>(
    ctx: crate::Context<'_, U, E>,
    command_name: &str,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    let command = ctx.framework().options().commands.iter().find(|command| {
        if command.name.eq_ignore_ascii_case(command_name) {
            return true;
        }
        if let Some(context_menu_name) = command.context_menu_name {
            if context_menu_name.eq_ignore_ascii_case(command_name) {
                return true;
            }
        }

        false
    });

    let reply = if let Some(command) = command {
        match command.help_text {
            Some(f) => f(),
            None => command
                .description
                .as_deref()
                .unwrap_or("No help available")
                .to_owned(),
        }
    } else {
        format!("No such command `{}`", command_name)
    };

    ctx.send(|b| b.content(reply).ephemeral(config.ephemeral))
        .await?;
    Ok(())
}

/// Writes a single line of the help menu, like "  /ping        Emits a ping message\n"
async fn append_command_line<U, E>(
    ctx: crate::Context<'_, U, E>,
    menu: &mut String,
    command: &crate::Command<U, E>,
    indent: &str,
) {
    if command.hide_in_help {
        return;
    }

    let prefix = if command.slash_action.is_some() {
        String::from("/")
    } else if command.prefix_action.is_some() {
        let options = &ctx.framework().options().prefix_options;

        match &options.prefix {
            Some(fixed_prefix) => fixed_prefix.clone(),
            None => match options.dynamic_prefix {
                Some(dynamic_prefix_callback) => {
                    match dynamic_prefix_callback(crate::PartialContext::from(ctx)).await {
                        Ok(Some(dynamic_prefix)) => dynamic_prefix,
                        // `String::new()` defaults to "" which is what we want
                        Err(_) | Ok(None) => String::new(),
                    }
                }
                None => String::new(),
            },
        }
    } else {
        // This is not a prefix or slash command, i.e. probably a context menu only command
        // which we will only show later
        return;
    };

    let total_command_name_length = prefix.chars().count() + command.name.chars().count();
    let padding = 12_usize.saturating_sub(total_command_name_length) + 1;
    let _ = writeln!(
        menu,
        "{}{}{}{}{}",
        indent,
        prefix,
        command.name,
        " ".repeat(padding),
        command.description.as_deref().unwrap_or("")
    );
}

/// Code for printing an overview of all commands (e.g. `~help`)
async fn help_all_commands<U, E>(
    ctx: crate::Context<'_, U, E>,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    let mut categories = crate::util::OrderedMap::<Option<&str>, Vec<&crate::Command<U, E>>>::new();
    for cmd in &ctx.framework().options().commands {
        categories
            .get_or_insert_with(cmd.category, Vec::new)
            .push(cmd);
    }

    let mut menu = String::from("```\n");
    for (category_name, commands) in categories {
        menu += category_name.unwrap_or("Commands");
        menu += ":\n";
        for command in commands {
            append_command_line(ctx, &mut menu, command, "  ").await;
            if config.show_subcommands {
                for command in &command.subcommands {
                    append_command_line(ctx, &mut menu, command, "    ").await;
                }
            }
        }
    }

    if config.show_context_menu_commands {
        menu += "\nContext menu commands:\n";

        for command in &ctx.framework().options().commands {
            let kind = match command.context_menu_action {
                Some(crate::ContextMenuCommandAction::User(_)) => "user",
                Some(crate::ContextMenuCommandAction::Message(_)) => "message",
                None => continue,
            };
            let name = command.context_menu_name.unwrap_or(&command.name);
            let _ = writeln!(menu, "  {} (on {})", name, kind);
        }
    }

    menu += "\n";
    menu += config.extra_text_at_bottom;
    menu += "\n```";

    ctx.send(|b| b.content(menu).ephemeral(config.ephemeral))
        .await?;
    Ok(())
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
///     let config = poise::builtins::HelpConfiguration {
///         extra_text_at_bottom: "\
/// Type ?help command for more info on a command.
/// You can edit your message to the bot and the bot will edit its response.",
///         ..Default::default()
///     };
///     poise::builtins::help(ctx, command.as_deref(), config).await?;
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
pub async fn help<U, E>(
    ctx: crate::Context<'_, U, E>,
    command: Option<&str>,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    match command {
        Some(command) => help_single_command(ctx, command, config).await,
        None => help_all_commands(ctx, config).await,
    }
}
