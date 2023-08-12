//! Tools for implementing automatic edit tracking, i.e. the bot automatically updating its response
//! when the user edits their command invocation message.

use crate::serenity_prelude as serenity;

/// A single cached command invocation
#[derive(Debug)]
struct CachedInvocation {
    /// User message that triggered this command invocation
    user_msg: serenity::Message,
    /// Associated bot response of this command invocation
    bot_response: Option<serenity::Message>,
    /// Whether the bot response should be deleted when the user deletes their message
    track_deletion: bool,
}

/// Stores messages and the associated bot responses in order to implement poise's edit tracking
/// feature.
#[derive(Debug)]
pub struct EditTracker {
    /// Duration after which cached messages can be purged
    max_duration: std::time::Duration,
    /// Cache, which stores invocation messages, and the corresponding bot response message if any
    // TODO: change to `OrderedMap<MessageId, (Message, Option<serenity::Message>)>`?
    cache: Vec<CachedInvocation>,
}

impl EditTracker {
    /// Create an edit tracker which tracks messages for the specified duration.
    ///
    /// Note: [`EditTracker`] will only purge messages outside the duration when [`Self::purge`]
    /// is called. If you supply the created [`EditTracker`] to [`crate::Framework`], the framework
    /// will take care of that by calling [`Self::purge`] periodically.
    pub fn for_timespan(duration: std::time::Duration) -> std::sync::Arc<std::sync::RwLock<Self>> {
        std::sync::Arc::new(std::sync::RwLock::new(Self {
            max_duration: duration,
            cache: Vec::new(),
        }))
    }

    /// Returns a copy of a newly up-to-date cached message, or a brand new generated message when
    /// not in cache. Also returns a bool with `true` if this message was previously tracked
    ///
    /// Returns None if the command shouldn't be re-run, e.g. if the message content wasn't edited
    pub fn process_message_update(
        &mut self,
        user_msg_update: &serenity::MessageUpdateEvent,
        ignore_edits_if_not_yet_responded: bool,
    ) -> Option<(serenity::Message, bool)> {
        match self
            .cache
            .iter_mut()
            .find(|invocation| invocation.user_msg.id == user_msg_update.id)
        {
            Some(invocation) => {
                if ignore_edits_if_not_yet_responded && invocation.bot_response.is_none() {
                    return None;
                }

                // If message content wasn't touched, don't re-run command
                // Note: this may be Some, but still identical to previous content. We want to
                // re-run the command in that case too; because that means the user explicitly
                // edited their message
                #[allow(clippy::question_mark)]
                if user_msg_update.content.is_none() {
                    return None;
                }

                user_msg_update.apply_to_message(&mut invocation.user_msg);
                Some((invocation.user_msg.clone(), true))
            }
            None => {
                if ignore_edits_if_not_yet_responded {
                    return None;
                }
                let mut user_msg = serenity::CustomMessage::new().build();
                user_msg_update.apply_to_message(&mut user_msg);
                Some((user_msg, false))
            }
        }
    }

    /// Removes this command invocation from the cache and returns the associated bot response,
    /// if the command invocation is cached, and it has an associated bot response, and the command
    /// is marked track_deletion
    pub fn process_message_delete(
        &mut self,
        deleted_message_id: serenity::MessageId,
    ) -> Option<serenity::Message> {
        let invocation = self.cache.remove(
            self.cache
                .iter()
                .position(|invocation| invocation.user_msg.id == deleted_message_id)?,
        );
        if invocation.track_deletion {
            invocation.bot_response
        } else {
            None
        }
    }

    /// Forget all of the messages that are older than the specified duration.
    pub fn purge(&mut self) {
        let max_duration = self.max_duration;
        self.cache.retain(|invocation| {
            let last_update = invocation
                .user_msg
                .edited_timestamp
                .unwrap_or(invocation.user_msg.timestamp);
            let age = serenity::Timestamp::now().unix_timestamp() - last_update.unix_timestamp();
            age < max_duration.as_secs() as i64
        });
    }

    /// Given a message by a user, find the corresponding bot response, if one exists and is cached.
    pub fn find_bot_response(
        &self,
        user_msg_id: serenity::MessageId,
    ) -> Option<&serenity::Message> {
        let invocation = self
            .cache
            .iter()
            .find(|invocation| invocation.user_msg.id == user_msg_id)?;
        invocation.bot_response.as_ref()
    }

    /// Notify the [`EditTracker`] that the given user message should be associated with the given
    /// bot response. Overwrites any previous associated bot response
    pub fn set_bot_response(
        &mut self,
        user_msg: &serenity::Message,
        bot_response: serenity::Message,
        track_deletion: bool,
    ) {
        if let Some(invocation) = self
            .cache
            .iter_mut()
            .find(|invocation| invocation.user_msg.id == user_msg.id)
        {
            invocation.bot_response = Some(bot_response);
        } else {
            self.cache.push(CachedInvocation {
                user_msg: user_msg.clone(),
                bot_response: Some(bot_response),
                track_deletion,
            });
        }
    }

    /// Store that this command is currently running; so that if the command is editing its own
    /// invocation message (e.g. removing embeds), we don't accidentally treat it as an
    /// `execute_untracked_edits` situation and start an infinite loop
    pub fn track_command(&mut self, user_msg: &serenity::Message, track_deletion: bool) {
        if !self
            .cache
            .iter()
            .any(|invocation| invocation.user_msg.id == user_msg.id)
        {
            self.cache.push(CachedInvocation {
                user_msg: user_msg.clone(),
                bot_response: None,
                track_deletion,
            });
        }
    }
}
