//! All functions to actually send a reply

use crate::serenity_prelude as serenity;

/// Send a message in the given context: normal message if prefix command, interaction response
/// if application command.
///
/// If you just want to send a string, use [`say_reply`].
///
/// Note: panics when called in an autocomplete context!
///
/// ```rust,no_run
/// # use poise::serenity_prelude as serenity;
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let ctx: poise::Context<'_, (), ()> = todo!();
/// ctx.send(poise::CreateReply::default()
///     .content("Works for slash and prefix commands")
///     .embed(serenity::CreateEmbed::new()
///         .title("Much versatile, very wow")
///         .description("I need more documentation ok?")
///     )
///     .ephemeral(true) // this one only applies in application commands though
/// ).await?;
/// # Ok(()) }
/// ```
pub async fn send_reply<'ret, 'arg, U: Send + Sync + 'static, E>(
    ctx: crate::Context<'ret, U, E>,
    builder: crate::CreateReply<'arg>,
) -> Result<crate::ReplyHandle<'ret>, serenity::Error>
where
    'ret: 'arg,
{
    Ok(match ctx {
        crate::Context::Prefix(ctx) => super::ReplyHandle(super::ReplyHandleInner::Prefix(
            crate::send_prefix_reply(ctx, builder).await?,
        )),
        crate::Context::Application(ctx) => crate::send_application_reply(ctx, builder).await?,
    })
}

/// Shorthand of [`send_reply`] for text-only messages
///
/// Note: panics when called in an autocomplete context!
pub async fn say_reply<U: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, U, E>,
    text: impl Into<String>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    send_reply(ctx, crate::CreateReply::default().content(text.into())).await
}

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::CommandInteraction::create_followup) is sent.
///
/// No-op if autocomplete context
pub async fn send_application_reply<'ret, 'arg, U: Send + Sync + 'static, E>(
    ctx: crate::ApplicationContext<'ret, U, E>,
    builder: crate::CreateReply<'arg>,
) -> Result<crate::ReplyHandle<'ret>, serenity::Error>
where
    'ret: 'arg,
{
    let builder = ctx.reply_builder(builder);

    if ctx.interaction_type == crate::CommandInteractionType::Autocomplete {
        return Ok(super::ReplyHandle(super::ReplyHandleInner::Autocomplete));
    }

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    let followup = if has_sent_initial_response {
        Some(Box::new({
            let builder = builder
                .to_slash_followup_response(serenity::CreateInteractionResponseFollowup::new());

            ctx.interaction.create_followup(ctx.http(), builder).await?
        }))
    } else {
        let builder =
            builder.to_slash_initial_response(serenity::CreateInteractionResponseMessage::new());

        ctx.interaction
            .create_response(
                ctx.http(),
                serenity::CreateInteractionResponse::Message(builder),
            )
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);

        None
    };

    Ok(super::ReplyHandle(super::ReplyHandleInner::Application {
        http: &ctx.serenity_context().http,
        interaction: ctx.interaction,
        followup,
    }))
}

/// Prefix-specific reply function. For more details, see [`crate::send_reply`].
pub async fn send_prefix_reply<'a, U: Send + Sync + 'static, E>(
    ctx: crate::PrefixContext<'a, U, E>,
    builder: crate::CreateReply<'a>,
) -> Result<Box<serenity::Message>, serenity::Error> {
    let builder = ctx.reply_builder(builder);

    // This must only return None when we _actually_ want to reuse the existing response! There are
    // no checks later
    let lock_edit_tracker = || {
        if let Some(edit_tracker) = &ctx.framework.options().prefix_options.edit_tracker {
            return Some(edit_tracker.write().unwrap());
        }
        None
    };

    let existing_response = if ctx.command.reuse_response {
        lock_edit_tracker()
            .as_mut()
            .and_then(|t| t.find_bot_response(ctx.msg.id))
            .cloned()
    } else {
        None
    };

    Ok(Box::new(if let Some(mut response) = existing_response {
        response
            .edit(ctx.serenity_context(), {
                // Reset the message. We don't want leftovers of the previous message (e.g. user
                // sends a message with `.content("abc")` in a track_edits command, and the edited
                // message happens to contain embeds, we don't want to keep those embeds)
                // (*f = Default::default() won't do)
                let b = serenity::EditMessage::new()
                    .content("")
                    .embeds(Vec::new())
                    .components(Vec::new())
                    .remove_all_attachments();

                builder.to_prefix_edit(b)
            })
            .await?;

        // If the entry still exists after the await, update it to the new contents
        // We don't check ctx.command.reuse_response because it's true anyways in this branch
        if let Some(mut edit_tracker) = lock_edit_tracker() {
            edit_tracker.set_bot_response(ctx.msg, response.clone(), ctx.command.track_deletion);
        }

        response
    } else {
        let new_response = ctx
            .msg
            .channel_id
            .send_message(ctx.serenity_context(), builder.to_prefix(ctx.msg.into()))
            .await?;
        // We don't check ctx.command.reuse_response because we need to store bot responses for
        // track_deletion too
        if let Some(track_edits) = &mut lock_edit_tracker() {
            track_edits.set_bot_response(ctx.msg, new_response.clone(), ctx.command.track_deletion);
        }

        new_response
    }))
}
