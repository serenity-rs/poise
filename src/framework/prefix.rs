use crate::serenity_prelude as serenity;

// Returns tuple of stripped prefix and rest of the message, if any prefix matches
async fn strip_prefix<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
) -> Option<(&'a str, &'a str)> {
    if let Some(dynamic_prefix) = this.options.prefix_options.dynamic_prefix {
        if let Some(prefix) = dynamic_prefix(ctx, msg, this.get_user_data().await).await {
            if msg.content.starts_with(&prefix) {
                return Some(msg.content.split_at(prefix.len()));
            }
        }
    }

    if let Some(prefix) = &this.options.prefix_options.prefix {
        if let Some(content) = msg.content.strip_prefix(prefix) {
            return Some((prefix, content));
        }
    }

    if let Some((prefix, content)) = this
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

    if let Some(dynamic_prefix) = this.options.prefix_options.stripped_dynamic_prefix {
        if let Some((prefix, content)) = dynamic_prefix(ctx, msg, this.get_user_data().await).await
        {
            return Some((prefix, content));
        }
    }

    if this.options.prefix_options.mention_as_prefix {
        // Mentions are either <@USER_ID> or <@!USER_ID>
        if let Some(stripped_content) = (|| {
            msg.content
                .strip_prefix("<@")?
                .trim_start_matches('!')
                .strip_prefix(&this.bot_id.0.to_string())?
                .strip_prefix('>')
        })() {
            let mention_prefix = &msg.content[..(msg.content.len() - stripped_content.len())];
            return Some((mention_prefix, stripped_content));
        }
    }

    None
}

/// Find a command within nested PrefixCommandMeta's by the user message string. Also returns
/// the arguments, i.e. the remaining string.
///
/// May throw an error if a command check fails
fn find_command<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
    prefix: &'a str,
    commands: &'a [crate::PrefixCommandMeta<U, E>],
    remaining_message: &'a str,
) -> crate::BoxFuture<
    'a,
    Result<
        Option<(&'a crate::PrefixCommandMeta<U, E>, &'a str)>,
        (E, crate::PrefixCommandErrorContext<'a, U, E>),
    >,
>
where
    U: Send + Sync,
{
    Box::pin(_find_command(
        this,
        ctx,
        msg,
        prefix,
        commands,
        remaining_message,
    ))
}

async fn _find_command<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
    prefix: &'a str,
    commands: &'a [crate::PrefixCommandMeta<U, E>],
    remaining_message: &'a str,
) -> Result<
    Option<(&'a crate::PrefixCommandMeta<U, E>, &'a str)>,
    (E, crate::PrefixCommandErrorContext<'a, U, E>),
>
where
    U: Send + Sync,
{
    let considered_equal = if this.options.prefix_options.case_insensitive_commands {
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

        let ctx = crate::PrefixContext {
            discord: ctx,
            msg,
            prefix,
            framework: this,
            data: this.get_user_data().await,
            command: Some(&command_meta.command),
        };

        // Make sure that user has required permissions
        if !super::check_required_permissions_and_owners_only(
            crate::Context::Prefix(ctx),
            command.id.required_permissions,
            command.id.owners_only,
        )
        .await
        {
            continue;
        }

        // Before running any checks, make sure the bot has the permissions it needs
        let missing_bot_permissions =
            super::check_missing_bot_permissions(ctx.into(), command.id.required_bot_permissions)
                .await;
        if !missing_bot_permissions.is_empty() {
            (ctx.framework.options().missing_bot_permissions_handler)(
                ctx.into(),
                missing_bot_permissions,
            )
            .await
            .map_err(|e| {
                (
                    e,
                    crate::PrefixCommandErrorContext {
                        ctx,
                        command,
                        location: crate::CommandErrorLocation::MissingBotPermissionsCallback,
                    },
                )
            })?;
            continue;
        }

        // Only continue if command checks returns true
        let checks_passing = (|| async {
            let global_check_passes = match &this.options.command_check {
                Some(check) => check(crate::Context::Prefix(ctx)).await?,
                None => true,
            };

            let command_specific_check_passes = match &command.options.check {
                Some(check) => check(ctx).await?,
                None => true,
            };

            Ok(global_check_passes && command_specific_check_passes)
        })()
        .await
        .map_err(|e| {
            (
                e,
                crate::PrefixCommandErrorContext {
                    command,
                    ctx,
                    location: crate::CommandErrorLocation::Check,
                },
            )
        })?;
        if !checks_passing {
            continue;
        }

        first_matching_command = Some(
            match find_command(
                this,
                ctx.discord,
                msg,
                prefix,
                &command_meta.subcommands,
                remaining_message,
            )
            .await?
            {
                Some((subcommand_meta, remaining_message)) => (subcommand_meta, remaining_message),
                None => (command_meta, remaining_message),
            },
        );
        break;
    }

    Ok(first_matching_command)
}

/// Manually dispatches a message with the prefix framework.
///
/// Returns:
/// - Ok(()) if a command was successfully dispatched and run
/// - Err(None) if no command was run but no error happened
/// - Err(Some(error: UserError)) if any user code yielded an error
pub async fn dispatch_message<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
    triggered_by_edit: bool,
    previously_tracked: bool,
) -> Result<(), Option<(E, crate::PrefixCommandErrorContext<'a, U, E>)>>
where
    U: Send + Sync,
{
    // Strip prefix and whitespace between prefix and command
    let (prefix, msg_content) = strip_prefix(this, ctx, msg).await.ok_or(None)?;
    let msg_content = msg_content.trim_start();

    // If we know our own ID, and the message author ID is our own, and we aren't supposed to
    // execute our own messages, THEN stop execution.
    if !this.options.prefix_options.execute_self_messages && this.bot_id == msg.author.id {
        return Err(None);
    }

    let (command_meta, args) = find_command(
        this,
        ctx,
        msg,
        prefix,
        &this.options.prefix_options.commands,
        msg_content,
    )
    .await
    .map_err(Some)?
    .ok_or(None)?;
    let command = &command_meta.command;

    // Check if we should disregard this invocation if it was triggered by an edit
    let should_execute_if_triggered_by_edit = command.options.track_edits
        || (!previously_tracked && this.options.prefix_options.execute_untracked_edits);
    if triggered_by_edit && !should_execute_if_triggered_by_edit {
        return Err(None);
    }

    let ctx = crate::PrefixContext {
        discord: ctx,
        msg,
        prefix,
        framework: this,
        data: this.get_user_data().await,
        command: Some(command),
    };

    let cooldowns = &command.id.cooldowns;
    let cooldown_left = cooldowns.lock().unwrap().get_wait_time(ctx.into());
    if let Some(cooldown_left) = cooldown_left {
        if let Some(callback) = ctx.framework.options().cooldown_hit {
            callback(ctx.into(), cooldown_left).await.map_err(|e| {
                Some((
                    e,
                    crate::PrefixCommandErrorContext {
                        ctx,
                        command,
                        location: crate::CommandErrorLocation::CooldownCallback,
                    },
                ))
            })?;
        }
        return Err(None);
    }
    cooldowns.lock().unwrap().start_cooldown(ctx.into());

    // Typing is broadcasted as long as this object is alive
    let _typing_broadcaster = if command.options.broadcast_typing {
        msg.channel_id.start_typing(&ctx.discord.http).ok()
    } else {
        None
    };

    (this.options.pre_command)(crate::Context::Prefix(ctx)).await;

    // Execute command
    let res = (command.action)(ctx, args).await.map_err(|e| {
        Some((
            e,
            crate::PrefixCommandErrorContext {
                ctx,
                command,
                location: crate::CommandErrorLocation::Check,
            },
        ))
    });

    (this.options.post_command)(crate::Context::Prefix(ctx)).await;

    res
}
