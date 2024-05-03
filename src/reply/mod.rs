//! Infrastructure for replying, i.e. sending a message in a command context

mod builder;
pub use builder::*;

mod send_reply;
pub use send_reply::*;

use crate::serenity_prelude as serenity;
use std::borrow::Cow;

/// Private enum so we can extend, split apart, or merge variants without breaking changes
#[derive(Clone)]
enum ReplyHandleInner<'a> {
    /// A reply sent to a prefix command, i.e. a normal standalone message
    Prefix(Box<serenity::Message>),
    /// An application command response
    Application {
        /// Serenity HTTP instance that can be used to request the interaction response message
        /// object
        http: &'a serenity::Http,
        /// Interaction which contains the necessary data to request the interaction response
        /// message object
        interaction: &'a serenity::CommandInteraction,
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
pub struct ReplyHandle<'a>(ReplyHandleInner<'a>);

impl ReplyHandle<'_> {
    /// Retrieve the message object of the sent reply.
    ///
    /// Note: to delete or edit, use [`ReplyHandle::delete()`] and [`ReplyHandle::edit()`] directly!
    /// Doing it via the methods from a Message object will fail for ephemeral messages
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
            } => interaction.get_response(http).await,
            Autocomplete => panic!("reply is a no-op in autocomplete context"),
        }
    }

    /// Retrieve the message object of the sent reply.
    ///
    /// Note: to delete or edit, use [`ReplyHandle::delete()`] and [`ReplyHandle::edit()`] directly!
    /// Doing it via the methods from a Message object will fail for ephemeral messages
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
            } => Ok(Cow::Owned(interaction.get_response(http).await?)),
            Autocomplete => panic!("reply is a no-op in autocomplete context"),
        }
    }

    /// Edits the message that this [`ReplyHandle`] points to
    // TODO: return the edited Message object?
    // TODO: should I eliminate the ctx parameter by storing it in self instead? Would infect
    //  ReplyHandle with <U, E> type parameters
    pub async fn edit<'a, U: Send + Sync + 'static, E>(
        &self,
        ctx: crate::Context<'a, U, E>,
        builder: CreateReply<'a>,
    ) -> Result<(), serenity::Error> {
        let reply = ctx.reply_builder(builder);

        match &self.0 {
            ReplyHandleInner::Prefix(msg) => {
                msg.clone()
                    .edit(ctx.serenity_context(), {
                        // Clear builder so that adding embeds or attachments won't add on top of
                        // the pre-edit items but replace them (which is apparently the more
                        // intuitive behavior). Notably, setting the builder to default doesn't
                        // mean the entire message is reset to empty: Discord only updates parts
                        // of the message that have had a modification specified
                        reply.to_prefix_edit(serenity::EditMessage::new())
                    })
                    .await?;
            }
            ReplyHandleInner::Application {
                http,
                interaction,
                followup: None,
            } => {
                let builder =
                    reply.to_slash_initial_response_edit(serenity::EditInteractionResponse::new());

                interaction.edit_response(http, builder).await?;
            }
            ReplyHandleInner::Application {
                http,
                interaction,
                followup: Some(msg),
            } => {
                let builder = reply
                    .to_slash_followup_response(serenity::CreateInteractionResponseFollowup::new());

                interaction.edit_followup(http, msg.id, builder).await?;
            }
            ReplyHandleInner::Autocomplete => panic!("reply is a no-op in autocomplete context"),
        }
        Ok(())
    }

    /// Deletes this message
    pub async fn delete<U: Send + Sync + 'static, E>(
        &self,
        ctx: crate::Context<'_, U, E>,
    ) -> Result<(), serenity::Error> {
        match &self.0 {
            ReplyHandleInner::Prefix(msg) => msg.delete(ctx.http(), None).await?,
            ReplyHandleInner::Application {
                http: _,
                interaction,
                followup,
            } => match followup {
                Some(followup) => {
                    interaction.delete_followup(ctx.http(), followup.id).await?;
                }
                None => {
                    interaction.delete_response(ctx.http()).await?;
                }
            },
            ReplyHandleInner::Autocomplete => panic!("delete is a no-op in autocomplete context"),
        }
        Ok(())
    }
}
