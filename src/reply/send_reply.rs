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
/// # async fn _test(ctx: poise::Context<'_, (), ()>) -> Result<(), Box<dyn std::error::Error>> {
/// ctx.send(poise::CreateReply::default()
///     .content("Works for slash and prefix commands")
///     .embed(serenity::CreateEmbed::default()
///         .title("Much versatile, very wow")
///         .description("I need more documentation ok?")
///     )
///     .ephemeral(true) // this one only applies in application commands though
/// ).await?;
/// # Ok(()) }
/// ```
pub async fn send_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: crate::CreateReply,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    Ok(match ctx {
        crate::Context::Prefix(ctx) => crate::ReplyHandle(super::ReplyHandleInner::Prefix(
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
    send_reply(ctx, crate::CreateReply::default().content(text.into())).await
}

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::CommandInteraction::create_followup) is sent.
///
/// No-op if autocomplete context
pub async fn send_application_reply<U, E>(
    ctx: crate::ApplicationContext<'_, U, E>,
    builder: crate::CreateReply,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    _send_application_reply(ctx, builder.complete_from_ctx(ctx.into())).await
}

/// private version of [`send_application_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_application_reply<U, E>(
    ctx: crate::ApplicationContext<'_, U, E>,
    data: crate::CreateReply,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    let interaction = match ctx.interaction {
        crate::CommandOrAutocompleteInteraction::Command(x) => x,
        crate::CommandOrAutocompleteInteraction::Autocomplete(_) => {
            return Ok(crate::ReplyHandle(super::ReplyHandleInner::Autocomplete))
        }
    };

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    let followup = if has_sent_initial_response {
        Some(Box::new(
            interaction
                .create_followup(ctx.discord, data.to_slash_followup_response())
                .await?,
        ))
    } else {
        interaction
            .create_response(
                ctx.discord,
                serenity::CreateInteractionResponse::Message(data.to_slash_initial_response()),
            )
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);

        None
    };

    Ok(crate::ReplyHandle(crate::ReplyHandleInner::Application {
        http: &ctx.discord.http,
        interaction,
        followup,
    }))
}

/// Prefix-specific reply function. For more details, see [`crate::send_reply`].
pub async fn send_prefix_reply<'att, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    builder: crate::CreateReply,
) -> Result<Box<serenity::Message>, serenity::Error> {
    _send_prefix_reply(ctx, builder.complete_from_ctx(ctx.into())).await
}

/// private version of [`send_prefix_reply`] that isn't generic over the builder to minimize monomorphization-related codegen bloat
async fn _send_prefix_reply<'a, U, E>(
    ctx: crate::PrefixContext<'_, U, E>,
    reply: crate::CreateReply,
) -> Result<Box<serenity::Message>, serenity::Error> {
    // This must only return None when we _actually_ want to reuse the existing response! There are
    // no checks later
    let lock_edit_tracker = || {
        if ctx.command.reuse_response {
            if let Some(edit_tracker) = &ctx.framework.options().prefix_options.edit_tracker {
                return Some(edit_tracker.write().unwrap());
            }
        }
        None
    };

    let existing_response = lock_edit_tracker()
        .as_mut()
        .and_then(|t| t.find_bot_response(ctx.msg.id))
        .cloned();

    Ok(Box::new(if let Some(mut response) = existing_response {
        response.edit(ctx.discord, reply.to_prefix_edit()).await?;

        // If the entry still exists after the await, update it to the new contents
        if let Some(mut edit_tracker) = lock_edit_tracker() {
            edit_tracker.set_bot_response(ctx.msg, response.clone());
        }

        response
    } else {
        let new_response = ctx
            .msg
            .channel_id
            .send_message(ctx.discord, reply.to_prefix(ctx.msg))
            .await?;
        if let Some(track_edits) = &mut lock_edit_tracker() {
            track_edits.set_bot_response(ctx.msg, new_response.clone());
        }

        new_response
    }))
}
