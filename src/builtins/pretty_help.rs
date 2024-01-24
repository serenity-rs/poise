//! Contains a built-in help command and surrounding infrastructure that uses embeds.

use crate::{serenity_prelude as serenity, CreateReply};
use std::fmt::Write as _;

/// Optional configuration for how the help message from [`pretty_help()`] looks
pub struct PrettyHelpConfiguration<'a> {
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
    /// Color of the Embed
    pub color: (u8, u8, u8),
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for PrettyHelpConfiguration<'_> {
    fn default() -> Self {
        Self {
            extra_text_at_bottom: "",
            ephemeral: true,
            show_context_menu_commands: false,
            show_subcommands: false,
            include_description: true,
            color: (0, 110, 51),
            __non_exhaustive: (),
        }
    }
}

/// A help command that works similarly to `builtin::help` butt outputs text in an embed.
///
pub async fn pretty_help<U, E>(
    ctx: crate::Context<'_, U, E>,
    command: Option<&str>,
    config: PrettyHelpConfiguration<'_>,
) -> Result<(), serenity::Error> {
    match command {
        Some(command) => pretty_help_single_command(ctx, command, config).await,
        None => pretty_help_all_commands(ctx, config).await,
    }
}

/// Printing an overview of all commands (e.g. `~help`)
async fn pretty_help_all_commands<U, E>(
    ctx: crate::Context<'_, U, E>,
    config: PrettyHelpConfiguration<'_>,
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

                if let Some(description) = cmd.description.as_deref() {
                    writeln!(buffer, "{}{}`: *{}*", prefix, name, description).ok();
                } else {
                    writeln!(buffer, "{}{}`.", prefix, name).ok();
                }

                if config.show_subcommands {
                    for sbcmd in &cmd.subcommands {
                        let name = sbcmd.context_menu_name.as_deref().unwrap_or(&sbcmd.name);
                        let prefix = format_cmd_prefix(sbcmd, &options_prefix);

                        if let Some(description) = sbcmd.description.as_deref() {
                            writeln!(buffer, "> {}{}`: *{}*", prefix, name, description).ok();
                        } else {
                            writeln!(buffer, "> {}{}`.", prefix, name).ok();
                        }
                    }
                }
            }
            if let Some((i, _)) = buffer.char_indices().nth(1024) {
                buffer.truncate(i);
            }
            (category.unwrap_or_default(), buffer, false)
        })
        .collect::<Vec<_>>();

    let embed = serenity::CreateEmbed::new()
        .title("Help")
        .fields(fields)
        .color(config.color)
        .footer(serenity::CreateEmbedFooter::new(
            config.extra_text_at_bottom,
        ));

    let reply = crate::CreateReply::default()
        .embed(embed)
        .ephemeral(config.ephemeral);

    ctx.send(reply).await?;

    Ok(())
}

/// Figures out which prefix a command should have
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
    config: PrettyHelpConfiguration<'_>,
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

    let mut invocations = Vec::new();
    let mut subprefix = None;

    if command.slash_action.is_some() {
        invocations.push(format!("`/{}`", command.name));
        subprefix = Some(format!("> `/{}`", command.name));
    }
    if command.prefix_action.is_some() {
        let prefix = super::help::get_prefix_from_options(ctx)
            .await
            // This can happen if the prefix is dynamic, and the callback fails
            // due to help being invoked with slash or context menu commands.
            .unwrap_or_else(|| String::from("<prefix>"));
        invocations.push(format!("`{}{}`", prefix, command.name));
        subprefix = subprefix.or(Some(format!("> `{}{}`", prefix, command.name)));
    }
    if command.context_menu_name.is_some() && command.context_menu_action.is_some() {
        let kind = match command.context_menu_action {
            Some(crate::ContextMenuCommandAction::User(_)) => "user",
            Some(crate::ContextMenuCommandAction::Message(_)) => "message",
            Some(crate::ContextMenuCommandAction::__NonExhaustive) | None => unreachable!(),
        };
        invocations.push(format!(
            "`{}` (on {})",
            command
                .context_menu_name
                .as_deref()
                .unwrap_or(&command.name),
            kind
        ));
        subprefix = subprefix.or(Some(String::from("> ")));
    }
    // At least one of the three if blocks should have triggered
    assert!(!invocations.is_empty());
    assert!(subprefix.is_some());

    let invocations = invocations
        .into_iter()
        .reduce(|x, y| format!("{x}\n{y}"))
        .map(|s| ("", s, false));

    let description = match (&command.description, &command.help_text) {
        (Some(description), Some(help_text)) if config.include_description => {
            format!("{}\n\n{}", description, help_text)
        }
        (_, Some(help_text)) => help_text.clone(),
        (Some(description), None) => description.clone(),
        (None, None) => "No help available".to_string(),
    };

    let parameters = command
        .parameters
        .iter()
        .map(|parameter| {
            let req = if parameter.required {
                "required"
            } else {
                "optional"
            };
            if let Some(description) = parameter.description.as_deref() {
                format!("`{}` ({}) *{} *.", parameter.name, req, description)
            } else {
                format!("`{}` ({}).", parameter.name, req)
            }
        })
        .reduce(|x, y| format!("{x}\n{y}"))
        .map(|s| ("Parameters", s, false));

    let sbcmds = command
        .subcommands
        .iter()
        .map(|sbcmd| {
            let prefix = format_cmd_prefix(sbcmd, &subprefix); // i have no idea about this really
            let name = sbcmd.context_menu_name.as_deref().unwrap_or(&sbcmd.name);
            if let Some(description) = sbcmd.description.as_deref() {
                format!("> {}{}`: *{} *", prefix, name, description)
            } else {
                format!("> {}{}`", prefix, name,)
            }
        })
        .reduce(|x, y| format!("{x}\n{y}"))
        .map(|s| ("Subcommands", s, false));

    let fields = invocations
        .into_iter()
        .chain(parameters.into_iter())
        .chain(sbcmds.into_iter());

    let embed = serenity::CreateEmbed::default()
        .description(description)
        .fields(fields);

    let reply = CreateReply::default()
        .embed(embed)
        .ephemeral(config.ephemeral);

    ctx.send(reply).await?;
    Ok(())
}
