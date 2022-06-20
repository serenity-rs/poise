//! Infrastructure for replying, i.e. sending a message in a command context

mod builder;
pub use builder::*;

mod send_reply;
pub use send_reply::*;

use crate::serenity_prelude as serenity;
use std::borrow::Cow;

/// Returned from [`send_reply()`] to retrieve the sent message object.
///
/// Discord sometimes returns the [`serenity::Message`] object directly, but sometimes you have to
/// request it manually. This enum abstracts over the two cases
#[derive(Clone)]
pub enum ReplyHandle<'a> {
    /// When sending a normal message or application command followup response, Discord returns the
    /// message object directly
    Known(Box<serenity::Message>),
    /// When sending an initial application command response, you need to request the message object
    /// seperately
    Unknown {
        /// Serenity HTTP instance that can be used to request the interaction response message
        /// object
        http: &'a serenity::Http,
        /// Interaction which contains the necessary data to request the interaction response
        /// message object
        interaction: &'a serenity::ApplicationCommandInteraction,
    },
    /// Reply was attempted to be sent in autocomplete context, resulting in a no-op. Calling
    /// methods on this variant will panic
    Autocomplete,
}

impl ReplyHandle<'_> {
    #[cold]
    #[track_caller]
    /// Panics for when the variant is autocomplete
    fn autocomplete_panic() -> ! {
        panic!("reply is a no-op in autocomplete context")
    }

    /// Retrieve the message object of the sent reply.
    ///
    /// If you don't need ownership of Message, you can use [`ReplyHandle::message`]
    ///
    /// Only needs to do an HTTP request in the application command response case
    pub async fn into_message(self) -> Result<serenity::Message, serenity::Error> {
        match self {
            Self::Known(msg) => Ok(*msg),
            Self::Unknown { http, interaction } => interaction.get_interaction_response(http).await,
            Self::Autocomplete => Self::autocomplete_panic(),
        }
    }

    /// Retrieve the message object of the sent reply.
    ///
    /// Returns a reference to the known Message object, or fetches the message from the discord API.
    pub async fn message(&self) -> Result<Cow<'_, serenity::Message>, serenity::Error> {
        match self {
            Self::Known(msg) => Ok(Cow::Borrowed(msg)),
            Self::Unknown { http, interaction } => Ok(Cow::Owned(
                interaction.get_interaction_response(http).await?,
            )),
            Self::Autocomplete => Self::autocomplete_panic(),
        }
    }

    /// Edits the message that this [`ReplyHandle`] points to
    // TODO: return the edited Message object?
    pub async fn edit<'att, U, E>(
        &self,
        ctx: crate::Context<'_, U, E>,
        builder: impl for<'a> FnOnce(&'a mut CreateReply<'att>) -> &'a mut CreateReply<'att>,
    ) -> Result<(), serenity::Error> {
        // TODO: deduplicate this block of code
        let mut reply = crate::CreateReply {
            ephemeral: ctx.command().ephemeral,
            allowed_mentions: ctx.framework().options().allowed_mentions.clone(),
            ..Default::default()
        };
        builder(&mut reply);
        if let Some(callback) = ctx.framework().options().reply_callback {
            callback(ctx, &mut reply);
        }

        match self {
            Self::Known(msg) => {
                msg.clone()
                    .edit(ctx.discord(), |b| {
                        reply.to_prefix_edit(b);
                        b
                    })
                    .await?;
            }
            Self::Unknown { http, interaction } => {
                interaction
                    .edit_original_interaction_response(http, |b| {
                        reply.to_slash_initial_response_edit(b);
                        b
                    })
                    .await?;
            }
            Self::Autocomplete => Self::autocomplete_panic(),
        }
        Ok(())
    }
}
