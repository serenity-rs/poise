//! Infrastructure for replying, i.e. sending a message in a command context

mod builder;
pub use builder::*;

mod send_reply;
pub use send_reply::*;

use crate::serenity_prelude as serenity;
use std::borrow::Cow;

/// Private enum so we can extend, split apart, or merge variants without breaking changes
#[derive(Clone)]
pub(super) enum ReplyHandleInner<'a> {
    /// A reply sent to a prefix command, i.e. a normal standalone message
    Prefix(Box<serenity::Message>),
    /// An application command response
    Application {
        /// Serenity HTTP instance that can be used to request the interaction response message
        /// object
        http: &'a serenity::Http,
        /// Interaction which contains the necessary data to request the interaction response
        /// message object
        interaction: &'a serenity::ApplicationCommandInteraction,
        /// If this is a followup response, the Message object (which Discord only returns for
        /// followup responses, not initial)
        followup: Option<Box<serenity::Message>>,
    },
    /// Reply was attempted to be sent in autocomplete context, resulting in a no-op. Calling
    /// methods on this variant will panic
    Autocomplete,
}

/// Returned from [`send_reply()`] to operate on the sent message
///
/// Discord sometimes returns the [`serenity::Message`] object directly, but sometimes you have to
/// request it manually. This enum abstracts over the two cases
#[derive(Clone)]
pub struct ReplyHandle<'a>(pub(super) ReplyHandleInner<'a>);

impl ReplyHandle<'_> {
    /// Retrieve the message object of the sent reply.
    ///
    /// If you don't need ownership of Message, you can use [`ReplyHandle::message`]
    ///
    /// Only needs to do an HTTP request in the application command response case
    pub async fn into_message(self) -> Result<serenity::Message, serenity::Error> {
        use ReplyHandleInner::*;
        match self.0 {
            Prefix(msg)
            | Application {
                followup: Some(msg),
                ..
            } => Ok(*msg),
            Application {
                http,
                interaction,
                followup: None,
            } => interaction.get_interaction_response(http).await,
            Autocomplete => panic!("reply is a no-op in autocomplete context"),
        }
    }

    /// Retrieve the message object of the sent reply.
    ///
    /// Returns a reference to the known Message object, or fetches the message from the discord API.
    ///
    /// To get an owned [`serenity::Message`], use [`Self::into_message()`]
    pub async fn message(&self) -> Result<Cow<'_, serenity::Message>, serenity::Error> {
        use ReplyHandleInner::*;
        match &self.0 {
            Prefix(msg)
            | Application {
                followup: Some(msg),
                ..
            } => Ok(Cow::Borrowed(msg)),
            Application {
                http,
                interaction,
                followup: None,
            } => Ok(Cow::Owned(
                interaction.get_interaction_response(http).await?,
            )),
            Autocomplete => panic!("reply is a no-op in autocomplete context"),
        }
    }

    /// Edits the message that this [`ReplyHandle`] points to
    // TODO: return the edited Message object?
    // TODO: should I eliminate the ctx parameter by storing it in self instead? Would infect
    //  ReplyHandle with <U, E> type parameters
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

        match &self.0 {
            ReplyHandleInner::Prefix(msg) => {
                msg.clone()
                    .edit(ctx.discord(), |b| {
                        reply.to_prefix_edit(b);
                        b
                    })
                    .await?;
            }
            ReplyHandleInner::Application {
                http,
                interaction,
                followup: None,
            } => {
                interaction
                    .edit_original_interaction_response(http, |b| {
                        reply.to_slash_initial_response_edit(b);
                        b
                    })
                    .await?;
            }
            ReplyHandleInner::Application {
                http,
                interaction,
                followup: Some(msg),
            } => {
                interaction
                    .edit_followup_message(http, msg.id, |b| {
                        reply.to_slash_followup_response(b);
                        b
                    })
                    .await?;
            }
            ReplyHandleInner::Autocomplete => panic!("reply is a no-op in autocomplete context"),
        }
        Ok(())
    }
}
