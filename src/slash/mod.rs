//! Stores everything specific to application commands.

mod structs;
pub use structs::*;

mod argument;
pub use argument::*;

use crate::serenity_prelude as serenity;

/// Sends the message, specified via [`crate::CreateReply`], to the interaction initial response
/// endpoint
fn send_as_initial_response(
    mut data: crate::CreateReply<'_>,
    allowed_mentions: Option<&serenity::CreateAllowedMentions>,
    f: &mut serenity::CreateInteractionResponseData,
) {

    let crate::CreateReply {
        content,
        embeds,
        attachments: _, // serenity doesn't support attachments in initial response yet
        components,
        ephemeral,
        allowed_mentions,
        reference_message: _, // can't reply to a message in interactions
    } = {
        if data.allowed_mentions.is_none() {
            data.allowed_mentions = allowed_mentions.map(|i| i.clone());
        }

        data
    };

    if let Some(content) = content {
        f.content(content);
    }
    f.embeds(embeds);
    if let Some(allowed_mentions) = allowed_mentions {
        f.allowed_mentions(|f| {
            *f = allowed_mentions.clone();
            f
        });
    }
    if let Some(components) = components {
        f.components(|f| {
            f.0 = components.0;
            f
        });
    }
    if ephemeral {
        f.flags(serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
    }
}

/// Sends the message, specified via [`crate::CreateReply`], to the interaction followup response
/// endpoint
fn send_as_followup_response<'a>(
    mut data: crate::CreateReply<'a>,
    allowed_mentions: Option<&serenity::CreateAllowedMentions>,
    f: &mut serenity::CreateInteractionResponseFollowup<'a>,
) {
    let crate::CreateReply {
        content,
        embeds,
        attachments,
        components,
        ephemeral,
        allowed_mentions,
        reference_message: _,
    } = {
        if data.allowed_mentions.is_none() {
            data.allowed_mentions = allowed_mentions.map(|i| i.clone());
        }

        data
    };

    if let Some(content) = content {
        f.content(content);
    }
    f.embeds(embeds);
    if let Some(components) = components {
        f.components(|c| {
            c.0 = components.0;
            c
        });
    }
    if let Some(allowed_mentions) = allowed_mentions {
        f.allowed_mentions(|f| {
            *f = allowed_mentions.clone();
            f
        });
    }
    if ephemeral {
        f.flags(serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL);
    }
    f.add_files(attachments);
}

/// Sends the message, specified via [`crate::CreateReply`], to the interaction initial response
/// edit endpoint
fn send_as_edit<'a>(
    mut data: crate::CreateReply<'a>,
    allowed_mentions: Option<&serenity::CreateAllowedMentions>,
    f: &mut serenity::EditInteractionResponse,
) {
    let crate::CreateReply {
        content,
        embeds,
        attachments: _, // no support for attachment edits in serenity yet
        components,
        ephemeral: _, // can't edit ephemerality in retrospect
        allowed_mentions,
        reference_message: _,
    } = {
        if data.allowed_mentions.is_none() {
            data.allowed_mentions = allowed_mentions.map(|i| i.clone());
        }

        data
    };

    if let Some(content) = content {
        f.content(content);
    }
    f.set_embeds(embeds);
    if let Some(components) = components {
        f.components(|c| {
            c.0 = components.0;
            c
        });
    }
    if let Some(allowed_mentions) = allowed_mentions {
        f.allowed_mentions(|f| {
            *f = allowed_mentions.clone();
            f
        });
    }
}

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::ApplicationCommandInteraction::create_followup_message) is sent.
///
/// No-op if autocomplete context
pub async fn send_application_reply<'a, U, E>(
    ctx: ApplicationContext<'_, U, E>,
    builder: impl for<'b> FnOnce(&'b mut crate::CreateReply<'a>) -> &'b mut crate::CreateReply<'a>,
) -> Result<(), serenity::Error> {
    let interaction = match ctx.interaction {
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => return Ok(()),
    };

    let mut data = crate::CreateReply {
        ephemeral: ctx.command.ephemeral,
        ..Default::default()
    };
    builder(&mut data);

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    let allowed_mentions = ctx.framework.options().allowed_mentions.as_ref();
    if has_sent_initial_response {
        if ctx.command.reuse_response {
            interaction
                .edit_original_interaction_response(ctx.discord, |f| {
                    send_as_edit(data, allowed_mentions, f);
                    f
                })
                .await?;
        } else {
            interaction
                .create_followup_message(ctx.discord, |f| {
                    send_as_followup_response(data, allowed_mentions, f);
                    f
                })
                .await?;
        }
    } else {
        interaction
            .create_interaction_response(ctx.discord, |r| {
                r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|f| {
                        send_as_initial_response(data, allowed_mentions, f);
                        f
                    })
            })
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    Ok(())
}
