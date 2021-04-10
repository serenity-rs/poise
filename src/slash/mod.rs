//! Stores everything specific to slash commands.

mod structs;
pub use structs::*;

mod argument;
pub use argument::*;

use crate::serenity_prelude as serenity;

pub async fn send_slash_reply<U, E>(
    ctx: SlashContext<'_, U, E>,
    builder: impl FnOnce(&mut crate::CreateReply) -> &mut crate::CreateReply,
) -> Result<(), serenity::Error> {
    let mut reply = crate::CreateReply::default();
    builder(&mut reply);

    ctx.interaction
        .create_interaction_response(ctx.discord, |r| {
            r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|r| {
                    if let Some(content) = reply.content {
                        r.content(content);
                    }
                    if let Some(embed) = reply.embed {
                        r.embed(|e| {
                            *e = embed;
                            e
                        });
                    }
                    r
                })
        })
        .await?;

    Ok(())
}

pub async fn say_slash_reply<U, E>(
    ctx: SlashContext<'_, U, E>,
    text: String,
) -> Result<(), serenity::Error> {
    send_slash_reply(ctx, |m| m.content(text)).await
}
