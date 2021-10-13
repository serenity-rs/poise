use crate::serenity_prelude as serenity;

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

// Returns message with (only) bot prefix removed, if it matches
async fn strip_prefix<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
) -> Option<&'a str> {
    if let Some(dynamic_prefix) = this.options.prefix_options.dynamic_prefix {
        if let Some(prefix) = dynamic_prefix(ctx, msg, this.get_user_data().await).await {
            if let Some(content) = msg.content.strip_prefix(&prefix) {
                return Some(content);
            }
        }
    }

    if let Some(prefix) = &this.options.prefix_options.prefix {
        if let Some(content) = msg.content.strip_prefix(prefix) {
            return Some(content);
        }
    }

    if let Some(content) = this
        .options
        .prefix_options
        .additional_prefixes
        .iter()
        .find_map(|prefix| match prefix {
            crate::Prefix::Literal(prefix) => msg.content.strip_prefix(prefix),
            crate::Prefix::Regex(prefix) => {
                let regex_match = prefix.find(&msg.content)?;
                if regex_match.start() == 0 {
                    Some(&msg.content[regex_match.end()..])
                } else {
                    None
                }
            }
        })
    {
        return Some(content);
    }

    if let Some(dynamic_prefix) = this.options.prefix_options.stripped_dynamic_prefix {
        if let Some(content) = dynamic_prefix(ctx, msg, this.get_user_data().await).await {
            return Some(content);
        }
    }

    if this.options.prefix_options.mention_as_prefix {
        // Mentions are either <@USER_ID> or <@!USER_ID>
        let stripped_mention_prefix = || {
            msg.content
                .strip_prefix("<@")?
                .trim_start_matches('!')
                .strip_prefix(&this.bot_id.0.to_string())?
                .strip_prefix('>')
        };
        if let Some(content) = stripped_mention_prefix() {
            return Some(content);
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
    Box::pin(_find_command(this, ctx, msg, commands, remaining_message))
}

async fn _find_command<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    msg: &'a serenity::Message,
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

        let prefix_ctx = crate::PrefixContext {
            discord: ctx,
            msg,
            framework: this,
            data: this.get_user_data().await,
            command: Some(&command_meta.command),
        };

        // Make sure that user has required permissions
        if !super::check_required_permissions_and_owners_only(
            crate::Context::Prefix(prefix_ctx),
            command.options.required_permissions,
            command.options.owners_only,
        )
        .await
        {
            continue;
        }

        // Only continue if command check returns true
        let command_check = command
            .options
            .check
            .unwrap_or(this.options.prefix_options.command_check);
        let check_passes = command_check(prefix_ctx).await.map_err(|e| {
            (
                e,
                crate::PrefixCommandErrorContext {
                    command,
                    ctx: prefix_ctx,
                    while_checking: true,
                },
            )
        })?;
        if !check_passes {
            continue;
        }

        first_matching_command = Some(
            if let Some((subcommand_meta, remaining_message)) =
                find_command(this, ctx, msg, &command_meta.subcommands, remaining_message).await?
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
) -> Result<(), Option<(E, crate::PrefixCommandErrorContext<'a, U, E>)>>
where
    U: Send + Sync,
{
    // Strip prefix and whitespace between prefix and command
    let msg_content = strip_prefix(this, ctx, msg).await.ok_or(None)?.trim_start();

    // If we know our own ID, and the message author ID is our own, and we aren't supposed to
    // execute our own messages, THEN stop execution.
    if !this.options.prefix_options.execute_self_messages && this.bot_id == msg.author.id {
        return Err(None);
    }

    let (command_meta, args) = find_command(
        this,
        ctx,
        msg,
        &this.options.prefix_options.commands,
        msg_content,
    )
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
        .unwrap_or(&this.options.prefix_options.broadcast_typing)
    {
        crate::BroadcastTypingBehavior::None => None,
        crate::BroadcastTypingBehavior::WithDelay(delay) => {
            Some(DelayedTyping::start(&ctx.http, msg.channel_id, *delay))
        }
    };

    let ctx = crate::PrefixContext {
        discord: ctx,
        msg,
        framework: this,
        data: this.get_user_data().await,
        command: Some(command),
    };

    (this.options.pre_command)(crate::Context::Prefix(ctx)).await;

    // Execute command
    (command.action)(ctx, args).await.map_err(|e| {
        Some((
            e,
            crate::PrefixCommandErrorContext {
                ctx,
                command,
                while_checking: false,
            },
        ))
    })
}
