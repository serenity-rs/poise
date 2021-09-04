//! Stores everything specific to application commands.

mod structs;
pub use structs::*;

mod argument;
pub use argument::*;

use crate::serenity_prelude as serenity;

/// Send a response to an interaction (slash command or context menu command invocation).
///
/// If a response to this interaction has already been sent, a
/// [followup](serenity::model::interactions::Interaction::create_followup_message) is sent.
pub async fn send_application_reply<U, E>(
    ctx: ApplicationContext<'_, U, E>,
    builder: impl for<'a, 'b> FnOnce(&'a mut crate::CreateReply<'b>) -> &'a mut crate::CreateReply<'b>,
) -> Result<(), serenity::Error> {
    let mut reply = crate::CreateReply {
        ephemeral: ctx.command.options().ephemeral,
        ..Default::default()
    };
    builder(&mut reply);
    let crate::CreateReply {
        content,
        embed,
        attachments,
        components,
        ephemeral,
    } = reply;

    let has_sent_initial_response = ctx
        .has_sent_initial_response
        .load(std::sync::atomic::Ordering::SeqCst);

    if has_sent_initial_response {
        ctx.interaction
            .create_followup_message(ctx.discord, |f| {
                if let Some(content) = content {
                    f.content(content);
                }
                if let Some(embed) = embed {
                    f.add_embed(embed);
                }
                f.add_files(attachments);
                f
            })
            .await?;
    } else {
        ctx.interaction
            .create_interaction_response(ctx.discord, |r| {
                r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|r| {
                        if let Some(content) = content {
                            r.content(content);
                        }
                        if let Some(embed) = embed {
                            r.add_embed(embed);
                        }
                        if let Some(allowed_mentions) = &ctx.framework.options().allowed_mentions {
                            r.allowed_mentions(|m| {
                                *m = allowed_mentions.clone();
                                m
                            });
                        }
                        if let Some(components) = components {
                            r.components(|c| {
                                c.0 = components.0;
                                c
                            });
                        }
                        if ephemeral {
                            r.flags(
                                serenity::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL,
                            );
                        }
                        r
                    })
            })
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    Ok(())
}
