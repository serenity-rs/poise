//! Contains the built-in help command and surrounding infrastructure

use crate::{serenity_prelude as serenity, CreateReply};
use std::{borrow::Cow, fmt::Write as _};

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
    /// Whether to include [`crate::Command::description`] (above [`crate::Command::help_text`]).
    pub include_description: bool,
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
            include_description: true,
            __non_exhaustive: (),
        }
    }
}

/// Convenience function to align descriptions behind commands
struct TwoColumnList(Vec<(String, Option<String>)>);

impl TwoColumnList {
    /// Creates a new [`TwoColumnList`]
    fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a line that needs the padding between the columns
    fn push_two_colums(&mut self, command: String, description: String) {
        self.0.push((command, Some(description)));
    }

    /// Add a line that doesn't influence the first columns's width
    fn push_heading(&mut self, category: &str) {
        if !self.0.is_empty() {
            self.0.push(("".to_string(), None));
        }
        let mut category = category.to_string();
        category += ":";
        self.0.push((category, None));
    }

    /// Convert the list into a string with aligned descriptions
    fn into_string(self) -> String {
        let longest_command = self
            .0
            .iter()
            .filter_map(|(command, description)| {
                if description.is_some() {
                    Some(command.len())
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);
        let mut text = String::new();
        for (command, description) in self.0 {
            if let Some(description) = description {
                let padding = " ".repeat(longest_command - command.len() + 3);
                writeln!(text, "{}{}{}", command, padding, description).unwrap();
            } else {
                writeln!(text, "{}", command).unwrap();
            }
        }
        text
    }
}

/// Get the prefix from options
pub(super) async fn get_prefix_from_options<U: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, U, E>,
) -> Option<Cow<'_, str>> {
    let options = &ctx.framework().options().prefix_options;
    match &options.prefix {
        Some(fixed_prefix) => Some(fixed_prefix.clone()),
        None => match options.dynamic_prefix {
            Some(dynamic_prefix_callback) => {
                match dynamic_prefix_callback(crate::PartialContext::from(ctx)).await {
                    Ok(Some(dynamic_prefix)) => Some(dynamic_prefix),
                    _ => None,
                }
            }
            None => None,
        },
    }
}

/// Format context menu command name
fn format_context_menu_name<U, E>(command: &crate::Command<U, E>) -> Option<String> {
    let kind = match command.context_menu_action {
        Some(crate::ContextMenuCommandAction::User(_)) => "user",
        Some(crate::ContextMenuCommandAction::Message(_)) => "message",
        Some(crate::ContextMenuCommandAction::__NonExhaustive) => unreachable!(),
        None => return None,
    };
    Some(format!(
        "{} (on {})",
        command
            .context_menu_name
            .as_deref()
            .unwrap_or(&command.name),
        kind
    ))
}

/// Code for printing help of a specific command (e.g. `~help my_command`)
async fn help_single_command<U: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, U, E>,
    command_name: &str,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    let commands = &ctx.framework().options().commands;
    // Try interpret the command name as a context menu command first
    let mut command = commands.iter().find(|command| {
        if let Some(context_menu_name) = &command.context_menu_name {
            if context_menu_name.eq_ignore_ascii_case(command_name) {
                return true;
            }
        }
        false
    });
    // Then interpret command name as a normal command (possibly nested subcommand)
    if command.is_none() {
        if let Some((c, _, _)) = crate::find_command(commands, command_name, true, &mut vec![]) {
            command = Some(c);
        }
    }

    let reply = if let Some(command) = command {
        let mut invocations = Vec::new();
        let mut subprefix = None;
        if command.slash_action.is_some() {
            invocations.push(format!("`/{}`", command.name));
            subprefix = Some(format!("  /{}", command.name));
        }
        if command.prefix_action.is_some() {
            let prefix = match get_prefix_from_options(ctx).await {
                Some(prefix) => prefix,
                // None can happen if the prefix is dynamic, and the callback
                // fails due to help being invoked with slash or context menu
                // commands. Not sure there's a better way to handle this.
                None => Cow::Borrowed("<prefix>"),
            };
            invocations.push(format!("`{}{}`", prefix, command.name));
            if subprefix.is_none() {
                subprefix = Some(format!("  {}{}", prefix, command.name));
            }
        }
        if command.context_menu_name.is_some() && command.context_menu_action.is_some() {
            // Since command.context_menu_action is Some, this unwrap is safe
            invocations.push(format_context_menu_name(command).unwrap());
            if subprefix.is_none() {
                subprefix = Some(String::from("  "));
            }
        }
        // At least one of the three if blocks should have triggered
        assert!(subprefix.is_some());
        assert!(!invocations.is_empty());
        let invocations = invocations.join("\n");

        let mut text = match (&command.description, &command.help_text) {
            (Some(description), Some(help_text)) => {
                if config.include_description {
                    format!("{}\n\n{}", description, help_text)
                } else {
                    help_text.clone()
                }
            }
            (Some(description), None) => description.to_owned(),
            (None, Some(help_text)) => help_text.clone(),
            (None, None) => "No help available".to_string(),
        };
        if !command.parameters.is_empty() {
            text += "\n\n```\nParameters:\n";
            let mut parameterlist = TwoColumnList::new();
            for parameter in &command.parameters {
                let name = parameter.name.clone();
                let description = parameter.description.as_deref().unwrap_or("");
                let description = format!(
                    "({}) {}",
                    if parameter.required {
                        "required"
                    } else {
                        "optional"
                    },
                    description,
                );
                parameterlist.push_two_colums(name, description);
            }
            text += &parameterlist.into_string();
            text += "```";
        }
        if !command.subcommands.is_empty() {
            text += "\n\n```\nSubcommands:\n";
            let mut commandlist = TwoColumnList::new();
            // Subcommands can exist on context menu commands, but there's no
            // hierarchy in the menu, so just display them as a list without
            // subprefix.
            preformat_subcommands(
                &mut commandlist,
                command,
                &subprefix.unwrap_or_else(|| String::from("  ")),
            );
            text += &commandlist.into_string();
            text += "```";
        }
        format!("**{}**\n\n{}", invocations, text)
    } else {
        format!("No such command `{}`", command_name)
    };

    let reply = CreateReply::default()
        .content(reply)
        .ephemeral(config.ephemeral);

    ctx.send(reply).await?;
    Ok(())
}

/// Recursively formats all subcommands
fn preformat_subcommands<U, E>(
    commands: &mut TwoColumnList,
    command: &crate::Command<U, E>,
    prefix: &str,
) {
    let as_context_command = command.slash_action.is_none() && command.prefix_action.is_none();
    for subcommand in &command.subcommands {
        let command = if as_context_command {
            let name = format_context_menu_name(subcommand);
            if name.is_none() {
                continue;
            };
            name.unwrap()
        } else {
            format!("{} {}", prefix, subcommand.name)
        };
        let description = subcommand.description.as_deref().unwrap_or("").to_string();
        commands.push_two_colums(command, description);
        // We could recurse here, but things can get cluttered quickly.
        // Instead, we show (using this function) subsubcommands when
        // the user asks for help on the subcommand.
    }
}

/// Preformat lines (except for padding,) like `("  /ping", "Emits a ping message")`
fn preformat_command<U, E>(
    commands: &mut TwoColumnList,
    config: &HelpConfiguration<'_>,
    command: &crate::Command<U, E>,
    indent: &str,
    options_prefix: Option<&str>,
) {
    let prefix = if command.slash_action.is_some() {
        String::from("/")
    } else if command.prefix_action.is_some() {
        options_prefix.map(String::from).unwrap_or_default()
    } else {
        // This is not a prefix or slash command, i.e. probably a context menu only command
        // This should have been filtered out in `generate_all_commands`
        unreachable!();
    };

    let prefix = format!("{}{}{}", indent, prefix, command.name);
    commands.push_two_colums(
        prefix.clone(),
        command.description.as_deref().unwrap_or("").to_string(),
    );
    if config.show_subcommands {
        preformat_subcommands(commands, command, &prefix)
    }
}

/// Create help text for `help_all_commands`
///
/// This is a separate function so we can have tests for it
async fn generate_all_commands<U: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, U, E>,
    config: &HelpConfiguration<'_>,
) -> Result<String, serenity::Error> {
    let mut categories = crate::util::OrderedMap::<Option<&str>, Vec<&crate::Command<U, E>>>::new();
    for cmd in &ctx.framework().options().commands {
        categories
            .get_or_insert_with(cmd.category.as_deref(), Vec::new)
            .push(cmd);
    }

    let options_prefix = get_prefix_from_options(ctx).await;

    let mut menu = String::from("```\n");

    let mut commandlist = TwoColumnList::new();
    for (category_name, commands) in categories {
        let commands = commands
            .into_iter()
            .filter(|cmd| {
                !cmd.hide_in_help && (cmd.prefix_action.is_some() || cmd.slash_action.is_some())
            })
            .collect::<Vec<_>>();
        if commands.is_empty() {
            continue;
        }
        commandlist.push_heading(category_name.unwrap_or("Commands"));
        for command in commands {
            preformat_command(
                &mut commandlist,
                config,
                command,
                "  ",
                options_prefix.as_deref(),
            );
        }
    }
    menu += &commandlist.into_string();

    if config.show_context_menu_commands {
        menu += "\nContext menu commands:\n";

        for command in &ctx.framework().options().commands {
            let name = format_context_menu_name(command);
            if name.is_none() {
                continue;
            };
            let _ = writeln!(menu, "  {}", name.unwrap());
        }
    }

    menu += "\n";
    menu += config.extra_text_at_bottom;
    menu += "\n```";

    Ok(menu)
}

/// Code for printing an overview of all commands (e.g. `~help`)
async fn help_all_commands<U: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, U, E>,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    let menu = generate_all_commands(ctx, &config).await?;
    let reply = CreateReply::default()
        .content(menu)
        .ephemeral(config.ephemeral);

    ctx.send(reply).await?;
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
pub async fn help<U: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, U, E>,
    command: Option<&str>,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    match command {
        Some(command) => help_single_command(ctx, command, config).await,
        None => help_all_commands(ctx, config).await,
    }
}
