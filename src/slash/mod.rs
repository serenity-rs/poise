//! Stores everything specific to application commands.

mod structs;
pub use structs::*;

mod argument;
pub use argument::*;

use crate::serenity_prelude as serenity;

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::ApplicationCommandInteraction::create_followup_message) is sent.
///
/// No-op if autocomplete context
pub async fn send_application_reply<U, E>(
    ctx: ApplicationContext<'_, U, E>,
    builder: impl for<'a, 'b> FnOnce(&'a mut crate::CreateReply<'b>) -> &'a mut crate::CreateReply<'b>,
) -> Result<crate::ReplyHandle<'_>, serenity::Error> {
    let interaction = match ctx.interaction {
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => {
            return Ok(crate::ReplyHandle::Autocomplete)
        }
    };

    let mut data = crate::CreateReply {
        ephemeral: ctx.command.ephemeral,
        allowed_mentions: ctx.framework.options().allowed_mentions.clone(),
        ..Default::default()
    };
    builder(&mut data);
    if let Some(callback) = ctx.framework.options().reply_callback {
        callback(ctx.into(), &mut data);
    }

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    Ok(if has_sent_initial_response {
        crate::ReplyHandle::Known(Box::new(
            interaction
                .create_followup_message(ctx.discord, |f| {
                    data.to_slash_followup_response(f);
                    f
                })
                .await?,
        ))
    } else {
        interaction
            .create_interaction_response(ctx.discord, |r| {
                r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|f| {
                        data.to_slash_initial_response(f);
                        f
                    })
            })
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);

        crate::ReplyHandle::Unknown {
            http: &ctx.discord.http,
            interaction,
        }
    })
}
