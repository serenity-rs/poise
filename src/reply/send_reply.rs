//! All functions to actually send a reply

use crate::serenity_prelude as serenity;

/// Send a message in the given context: normal message if prefix command, interaction response
/// if application command.
///
/// If you just want to send a string, use [`say_reply`] or [`reply_reply`].
///
/// Note: panics when called in an autocomplete context!
///
/// ```rust,no_run
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let ctx: poise::Context<'_, (), ()> = todo!();
/// ctx.send(|f| f
///     .content("Works for slash and prefix commands")
///     .embed(|f| f
///         .title("Much versatile, very wow")
///         .description("I need more documentation ok?")
///     )
///     .ephemeral(true) // this one only applies in application commands though
/// ).await?;
/// # Ok(()) }
/// ```
pub async fn send_reply<'att, U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
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
pub async fn say_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: impl Into<String>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    send_reply(ctx, |m| m.content(text.into())).await
}

/// Shorthand of [`send_reply`] for text-only messages, and reply to the reference message for [`crate::PrefixContext`]
///
/// for [`crate::ApplicationContext`], this is the same as [`say_reply`].
///
/// Note: panics when called in an autocomplete context!
pub async fn reply_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: impl Into<String>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    send_reply(ctx, |m| m.content(text.into()).reply(true)).await
}

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::ApplicationCommandInteraction::create_followup_message) is sent.
///
/// No-op if autocomplete context
pub async fn send_application_reply<'att, U, E>(
    ctx: crate::ApplicationContext<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    _send_application_reply(ctx, ctx.reply_builder(builder)).await
}

/// private version of [`send_application_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_application_reply<'a, U, E>(
    ctx: crate::ApplicationContext<'a, U, E>,
    data: crate::CreateReply<'_>,
) -> Result<crate::ReplyHandle<'a>, serenity::Error> {
    let interaction = match ctx.interaction {
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => {
            return Ok(super::ReplyHandle(super::ReplyHandleInner::Autocomplete))
        }
    };

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    let followup = if has_sent_initial_response {
        Some(Box::new(
            interaction
                .create_followup_message(ctx.serenity_context, |f| {
                    data.to_slash_followup_response(f);
                    f
                })
                .await?,
        ))
    } else {
        interaction
            .create_interaction_response(ctx.serenity_context, |r| {
                r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|f| {
                        data.to_slash_initial_response(f);
                        f
                    })
            })
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);

        None
    };

    Ok(super::ReplyHandle(super::ReplyHandleInner::Application {
        http: &ctx.serenity_context.http,
        interaction,
        followup,
    }))
}

/// Prefix-specific reply function. For more details, see [`crate::send_reply`].
pub async fn send_prefix_reply<'att, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    builder: impl for<'a> FnOnce(&'a mut crate::CreateReply<'att>) -> &'a mut crate::CreateReply<'att>,
) -> Result<Box<serenity::Message>, serenity::Error> {
    _send_prefix_reply(ctx, ctx.reply_builder(builder)).await
}

/// private version of [`send_prefix_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_prefix_reply<'a, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    reply: crate::CreateReply<'a>,
) -> Result<Box<serenity::Message>, serenity::Error> {
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
            .edit(ctx.serenity_context, |f| {
                // Reset the message. We don't want leftovers of the previous message (e.g. user
                // sends a message with `.content("abc")` in a track_edits command, and the edited
                // message happens to contain embeds, we don't want to keep those embeds)
                // (*f = Default::default() won't do)
                f.content("");
                f.set_embeds(Vec::new());
                f.components(|b| b);
                f.0.insert("attachments", serenity::json::json! { [] });

                reply.to_prefix_edit(f);
                f
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
            .send_message(ctx.serenity_context, |m| {
                reply.to_prefix(m, ctx.msg);
                m
            })
            .await?;
        // We don't check ctx.command.reuse_response because we need to store bot responses for
        // track_deletion too
        if let Some(track_edits) = &mut lock_edit_tracker() {
            track_edits.set_bot_response(ctx.msg, new_response.clone(), ctx.command.track_deletion);
        }

        new_response
    }))
}
