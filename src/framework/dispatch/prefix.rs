//! Dispatches incoming messages and message edits onto framework commands

use crate::serenity_prelude as serenity;

/// Checks if this message is a bot invocation by attempting to strip the prefix
///
/// Returns tuple of stripped prefix and rest of the message, if any prefix matches
async fn strip_prefix<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
) -> Option<(&'a str, &'a str)> {
    if let Some(dynamic_prefix) = framework.options.prefix_options.dynamic_prefix {
        let partial_ctx = crate::PartialContext {
            guild_id: msg.guild_id,
            channel_id: msg.channel_id,
            author: &msg.author,
            discord: ctx,
            framework,
            data: framework.user_data().await,
        };
        match dynamic_prefix(partial_ctx).await {
            Ok(prefix) => {
                if let Some(prefix) = prefix {
                    if msg.content.starts_with(&prefix) {
                        return Some(msg.content.split_at(prefix.len()));
                    }
                }
            }
            Err(error) => {
                (framework.options.on_error)(crate::FrameworkError::DynamicPrefix { error }).await;
            }
        }
    }

    if let Some(prefix) = &framework.options.prefix_options.prefix {
        if let Some(content) = msg.content.strip_prefix(prefix) {
            return Some((prefix, content));
        }
    }

    if let Some((prefix, content)) = framework
        .options
        .prefix_options
        .additional_prefixes
        .iter()
        .find_map(|prefix| match prefix {
            &crate::Prefix::Literal(prefix) => Some((prefix, msg.content.strip_prefix(prefix)?)),
            crate::Prefix::Regex(prefix) => {
                let regex_match = prefix.find(&msg.content)?;
                if regex_match.start() == 0 {
                    Some(msg.content.split_at(regex_match.end()))
                } else {
                    None
                }
            }
        })
    {
        return Some((prefix, content));
    }

    if let Some(dynamic_prefix) = framework.options.prefix_options.stripped_dynamic_prefix {
        match dynamic_prefix(ctx, msg, framework.user_data().await).await {
            Ok(result) => {
                if let Some((prefix, content)) = result {
                    return Some((prefix, content));
                }
            }
            Err(error) => {
                (framework.options.on_error)(crate::FrameworkError::DynamicPrefix { error }).await;
            }
        }
    }

    if framework.options.prefix_options.mention_as_prefix {
        if let Some(bot_id) = framework.bot_id.get() {
            // Mentions are either <@USER_ID> or <@!USER_ID>
            if let Some(stripped_content) = (|| {
                msg.content
                    .strip_prefix("<@")?
                    .trim_start_matches('!')
                    .strip_prefix(&bot_id.0.to_string())?
                    .strip_prefix('>')
            })() {
                let mention_prefix = &msg.content[..(msg.content.len() - stripped_content.len())];
                return Some((mention_prefix, stripped_content));
            }
        } else {
            log::warn!("bot_id not yet initialized");
        }
    }

    None
}

/// Find a command or subcommand within `&[Command]`, given a command invocation without a prefix.
/// Returns the verbatim command name string as well as the command arguments (i.e. the remaining
/// string).
///
/// ```rust
/// #[poise::command(prefix_command)] async fn command1(ctx: poise::Context<'_, (), ()>) -> Result<(), ()> { Ok(()) }
/// #[poise::command(prefix_command)] async fn command2(ctx: poise::Context<'_, (), ()>) -> Result<(), ()> { Ok(()) }
/// #[poise::command(prefix_command)] async fn command3(ctx: poise::Context<'_, (), ()>) -> Result<(), ()> { Ok(()) }
/// let commands = vec![
///     command1(),    
///     poise::Command {
///         subcommands: vec![command3()],
///         ..command2()
///     },
/// ];
///
/// assert_eq!(
///     poise::find_command(&commands, "command1 my arguments", false),
///     Some((&commands[0], "command1", "my arguments")),
/// );
/// assert_eq!(
///     poise::find_command(&commands, "command2 command3 my arguments", false),
///     Some((&commands[1].subcommands[0], "command3", "my arguments")),
/// );
/// assert_eq!(
///     poise::find_command(&commands, "CoMmAnD2 cOmMaNd99 my arguments", true),
///     Some((&commands[1], "CoMmAnD2", "cOmMaNd99 my arguments")),
/// );
pub fn find_command<'a, U, E>(
    commands: &'a [crate::Command<U, E>],
    remaining_message: &'a str,
    case_insensitive: bool,
) -> Option<(&'a crate::Command<U, E>, &'a str, &'a str)>
where
    U: Send + Sync,
{
    let string_equal = if case_insensitive {
        |a: &str, b: &str| a.eq_ignore_ascii_case(b)
    } else {
        |a: &str, b: &str| a == b
    };

    let (command_name, remaining_message) = {
        let mut iter = remaining_message.splitn(2, char::is_whitespace);
        (iter.next().unwrap(), iter.next().unwrap_or("").trim_start())
    };

    for command in commands {
        let primary_name_matches = string_equal(command.name, command_name);
        let alias_matches = command
            .aliases
            .iter()
            .any(|alias| string_equal(alias, command_name));
        if !primary_name_matches && !alias_matches {
            continue;
        }

        return Some(
            find_command(&command.subcommands, remaining_message, case_insensitive).unwrap_or((
                command,
                command_name,
                remaining_message,
            )),
        );
    }

    None
}

/// Manually dispatches a message with the prefix framework.
///
/// Returns:
/// - Ok(()) if a command was successfully dispatched and run
/// - Err(None) if no command was dispatched, for example if the message didn't contain a command or
///   the cooldown limits were reached
/// - Err(Some(error: UserError)) if any user code yielded an error
pub async fn dispatch_message<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
    triggered_by_edit: bool,
    previously_tracked: bool,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
) -> Result<(), Option<(crate::FrameworkError<'a, U, E>, &'a crate::Command<U, E>)>>
where
    U: Send + Sync,
{
    // Check if we're allowed to invoke from bot messages
    if msg.author.bot && framework.options.prefix_options.ignore_bots {
        return Err(None);
    }

    // Check if we're allowed to execute our own messages
    if let Some(&bot_id) = framework.bot_id.get() {
        if bot_id == msg.author.id && !framework.options.prefix_options.execute_self_messages {
            return Err(None);
        }
    } else {
        log::warn!("bot_id not yet initialized");
    }

    // Strip prefix and whitespace between prefix and command
    let (prefix, msg_content) = strip_prefix(framework, ctx, msg).await.ok_or(None)?;
    let msg_content = msg_content.trim_start();

    let (command, invoked_command_name, args) = find_command(
        &framework.options.commands,
        msg_content,
        framework.options.prefix_options.case_insensitive_commands,
    )
    .ok_or(None)?;
    let action = command.prefix_action.ok_or(None)?;

    // Check if we should disregard this invocation if it was triggered by an edit
    let should_execute_if_triggered_by_edit = command.invoke_on_edit
        || (!previously_tracked && framework.options.prefix_options.execute_untracked_edits);
    if triggered_by_edit && !should_execute_if_triggered_by_edit {
        return Err(None);
    }

    let ctx = crate::PrefixContext {
        discord: ctx,
        msg,
        prefix,
        invoked_command_name,
        args,
        framework,
        data: framework.user_data().await,
        command,
        invocation_data,
        __non_exhaustive: (),
    };

    super::common::check_permissions_and_cooldown(ctx.into(), command)
        .await
        .map_err(|e| Some((e, command)))?;

    // Typing is broadcasted as long as this object is alive
    let _typing_broadcaster = if command.broadcast_typing {
        msg.channel_id.start_typing(&ctx.discord.http).ok()
    } else {
        None
    };

    (framework.options.pre_command)(crate::Context::Prefix(ctx)).await;

    // Store that this command is currently running; so that if the invocation message is being
    // edited before a response message is registered, we don't accidentally treat it as an
    // execute_untracked_edits situation and start an infinite loop
    // Reported by vicky5124 https://discord.com/channels/381880193251409931/381912587505500160/897981367604903966
    if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
        edit_tracker.write().unwrap().track_command(ctx.msg);
    }

    // Execute command
    let action_result = (action)(ctx).await;
    super::common::trigger_cooldown_maybe(ctx.into(), &action_result);
    action_result.map_err(|e| Some((e, command)))?;

    (framework.options.post_command)(crate::Context::Prefix(ctx)).await;

    Ok(())
}
