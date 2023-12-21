use super::help::HelpConfiguration;

use crate::{serenity_prelude as serenity, CreateReply};
use std::fmt::Write as _;

/// A help command that works similarly to `builtin::help` butt outputs text in an embed.
///
pub async fn pretty_help<U, E>(
    ctx: crate::Context<'_, U, E>,
    command: Option<&str>,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    match command {
        Some(command) => help_single_command(ctx, command, config).await,
        None => pretty_help_all_commands(ctx, config).await,
    }
}

async fn pretty_help_all_commands<U, E>(
    ctx: crate::Context<'_, U, E>,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    let mut categories = crate::util::OrderedMap::new();
    let commands = ctx.framework().options().commands.iter().filter(|cmd| {
        !cmd.hide_in_help
            && (cmd.prefix_action.is_some()
                || cmd.slash_action.is_some()
                || (cmd.context_menu_action.is_some() && config.show_context_menu_commands))
    });

    for cmd in commands {
        categories
            .get_or_insert_with(cmd.category.as_deref(), Vec::new)
            .push(cmd);
    }

    let options_prefix = super::help::get_prefix_from_options(ctx).await;

    let fields = categories
        .into_iter()
        .filter(|(_, cmds)| !cmds.is_empty())
        .map(|(category, mut cmds)| {
            // get context menu items at the bottom
            cmds.sort_by_key(|cmd| cmd.slash_action.is_none() && cmd.prefix_action.is_none());

            let mut buffer = String::new();

            for cmd in cmds {
                let name = cmd.context_menu_name.as_deref().unwrap_or(&cmd.name);
                let prefix = format_cmd_prefix(cmd, &options_prefix);

                write!(
                    buffer,
                    "{}{}`: _{}_\n",
                    prefix,
                    name,
                    cmd.description.as_deref().unwrap_or_default()
                )
                .ok();

                if config.show_subcommands {
                    for sbcmd in &cmd.subcommands {
                        let name = sbcmd.context_menu_name.as_deref().unwrap_or(&sbcmd.name);
                        let prefix = format_cmd_prefix(sbcmd, &options_prefix);

                        write!(
                            buffer,
                            "> {}{}`: _{}_\n",
                            prefix,
                            name,
                            sbcmd.description.as_deref().unwrap_or_default()
                        )
                        .ok();
                    }
                }
            }
            (category.unwrap_or_default(), buffer, false)
        })
        .collect::<Vec<_>>();

    let embed = serenity::CreateEmbed::new()
        .title("Help")
        .fields(fields)
        .color((0, 110, 51))
        .footer(serenity::CreateEmbedFooter::new(
            config.extra_text_at_bottom,
        ));

    let reply = crate::CreateReply::default()
        .embed(embed)
        .ephemeral(config.ephemeral);

    ctx.send(reply).await?;

    Ok(())
}

fn format_cmd_prefix<U, E>(cmd: &crate::Command<U, E>, options_prefix: &Option<String>) -> String {
    if cmd.slash_action.is_some() {
        "`/".into()
    } else if cmd.prefix_action.is_some() {
        format!("`{}", options_prefix.as_deref().unwrap_or_default())
    } else if cmd.context_menu_action.is_some() {
        match cmd.context_menu_action {
            Some(crate::ContextMenuCommandAction::Message(_)) => "Message menu: `".into(),
            Some(crate::ContextMenuCommandAction::User(_)) => "User menu: `".into(),
            Some(crate::ContextMenuCommandAction::__NonExhaustive) | None => {
                unreachable!()
            }
        }
    } else {
        "`".into()
    }
}

/// Code for printing help of a specific command (e.g. `~help my_command`)
async fn pretty_help_single_command<U, E>(
    ctx: crate::Context<'_, U, E>,
    command_name: &str,
    config: HelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    let commands = &ctx.framework().options().commands;

    // Try interpret the command name as a context menu command first
    let command = commands
        .iter()
        .find(|cmd| {
            cmd.context_menu_name
                .as_ref()
                .is_some_and(|n| n.eq_ignore_ascii_case(command_name))
        })
        // Then interpret command name as a normal command (possibly nested subcommand)
        .or(crate::find_command(commands, command_name, true, &mut vec![]).map(|(c, _, _)| c));

    let Some(command) = command else {
        ctx.send(
            CreateReply::default()
                .content(format!("No such command `{}`", command_name))
                .ephemeral(config.ephemeral),
        )
        .await?;

        return Ok(());
    };

    let reply = {
        let mut invocations = Vec::new();
        let mut subprefix = None;
        if command.slash_action.is_some() {
            invocations.push(format!("`/{}`", command.name));
            subprefix = Some(format!("> /{}", command.name));
        }
        if command.prefix_action.is_some() {
            let prefix = super::help::get_prefix_from_options(ctx)
                .await
                // This can happen if the prefix is dynamic, and the callback fails
                // due to help being invoked with slash or context menu commands.
                .unwrap_or_else(|| String::from("<prefix>"));
            invocations.push(format!("`{}{}`", prefix, command.name));
            subprefix = subprefix.or(Some(format!("> {}{}", prefix, command.name)));
        }
        if command.context_menu_name.is_some() && command.context_menu_action.is_some() {
            // Since command.context_menu_action is Some, this unwrap is safe
            invocations.push(format_context_menu_name(command).unwrap());
            subprefix = subprefix.or(Some(String::from("> ")));
        }
        // At least one of the three if blocks should have triggered
        assert!(!invocations.is_empty());
        assert!(subprefix.is_some());

        let invocations = invocations.join("\n");
        let subprefix = subprefix.unwrap();

        let mut description = match (&command.description, &command.help_text) {
            (Some(description), Some(help_text)) if config.include_description => {
                format!("{}\n\n{}", description, help_text)
            }
            (_, Some(help_text)) => help_text.clone(),
            (Some(description), None) => description.clone(),
            (None, None) => "No help available".to_string(),
        };

        //
        // TODO
        //

        if !command.parameters.is_empty() {
            description += "\n\n```\nParameters:\n";
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
            description += &parameterlist.into_string();
            description += "```";
        }
        if !command.subcommands.is_empty() {
            description += "\n\n```\nSubcommands:\n";
            let mut commandlist = TwoColumnList::new();
            // Subcommands can exist on context menu commands, but there's no
            // hierarchy in the menu, so just display them as a list without
            // subprefix.
            preformat_subcommands(&mut commandlist, command, &subprefix);
            description += &commandlist.into_string();
            description += "```";
        }
        format!("**{}**\n\n{}", invocations, description)
    };

    let reply = CreateReply::default()
        .content(reply)
        .ephemeral(config.ephemeral);

    ctx.send(reply).await?;
    Ok(())
}

/// Convenience function to align descriptions behind commands
struct TwoColumnList(Vec<(String, Option<String>)>);

#[allow(unused)]
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
async fn help_single_command<U, E>(
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
            let prefix = match super::help::get_prefix_from_options(ctx).await {
                Some(prefix) => prefix,
                // None can happen if the prefix is dynamic, and the callback
                // fails due to help being invoked with slash or context menu
                // commands. Not sure there's a better way to handle this.
                None => String::from("<prefix>"),
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
