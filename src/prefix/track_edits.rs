//! Tools for implementing automatic edit tracking, i.e. the bot automatically updating its response
//! when the user edits their command invocation message.

use crate::serenity_prelude as serenity;

/// Updates the given message according to the update event
fn update_message(message: &mut serenity::Message, update: serenity::MessageUpdateEvent) {
    message.id = update.id;
    message.channel_id = update.channel_id;
    message.guild_id = update.guild_id;

    if let Some(kind) = update.kind {
        message.kind = kind;
    }
    if let Some(content) = update.content {
        message.content = content;
    }
    if let Some(tts) = update.tts {
        message.tts = tts;
    }
    if let Some(pinned) = update.pinned {
        message.pinned = pinned;
    }
    if let Some(timestamp) = update.timestamp {
        message.timestamp = timestamp;
    }
    if let Some(edited_timestamp) = update.edited_timestamp {
        message.edited_timestamp = Some(edited_timestamp);
    }
    if let Some(author) = update.author {
        message.author = author;
    }
    if let Some(mention_everyone) = update.mention_everyone {
        message.mention_everyone = mention_everyone;
    }
    if let Some(mentions) = update.mentions {
        message.mentions = mentions;
    }
    if let Some(mention_roles) = update.mention_roles {
        message.mention_roles = mention_roles;
    }
    if let Some(attachments) = update.attachments {
        message.attachments = attachments;
    }
    // if let Some(embeds) = update.embeds {
    //     message.embeds = embeds;
    // }
}

/// Stores messages and the associated bot responses in order to implement poise's edit tracking
/// feature.
#[derive(Debug)]
pub struct EditTracker {
    /// Duration after which cached messages can be purged
    max_duration: std::time::Duration,
    /// Cache, which associates user messages to the corresponding bot response message
    cache: Vec<(serenity::Message, serenity::Message)>,
}

impl EditTracker {
    /// Create an edit tracker which tracks messages for the specified duration.
    ///
    /// Note: [`EditTracker`] will only purge messages outside the duration when [`Self::purge`]
    /// is called. If you supply the created [`EditTracker`] to [`crate::Framework`], the framework
    /// will take care of that by calling [`Self::purge`] periodically.
    pub fn for_timespan(duration: std::time::Duration) -> std::sync::RwLock<Self> {
        std::sync::RwLock::new(Self {
            max_duration: duration,
            cache: Vec::new(),
        })
    }

    /// Returns a copy of a newly up-to-date cached message, or a brand new generated message when
    /// not in cache. Also returns a bool with `true` if this message was previously tracked
    ///
    /// Returns None if the command shouldn't be re-run, e.g. if the message content wasn't edited
    pub fn process_message_update(
        &mut self,
        user_msg_update: &serenity::MessageUpdateEvent,
        ignore_edit_tracker_cache: bool,
    ) -> Option<(serenity::Message, bool)> {
        match self
            .cache
            .iter_mut()
            .find(|(user_msg, _)| user_msg.id == user_msg_update.id)
        {
            Some((user_msg, _)) => {
                // If message content wasn't touched, don't re-run command
                // Note: this may be Some, but still identical to previous content. We want to
                // re-run the command in that case too; because that means the user explicitly
                // edited their message
                #[allow(clippy::question_mark)]
                if user_msg_update.content.is_none() {
                    return None;
                }

                update_message(user_msg, user_msg_update.clone());
                Some((user_msg.clone(), true))
            }
            None => {
                if !ignore_edit_tracker_cache {
                    let mut user_msg = serenity::CustomMessage::new().build();
                    update_message(&mut user_msg, user_msg_update.clone());
                    Some((user_msg, false))
                } else {
                    None
                }
            }
        }
    }

    /// Forget all of the messages that are older than the specified duration.
    pub fn purge(&mut self) {
        let max_duration = self.max_duration;
        self.cache.retain(|(user_msg, _)| {
            let last_update = user_msg.edited_timestamp.unwrap_or(user_msg.timestamp);
            if let Ok(age) = (chrono::Utc::now() - last_update).to_std() {
                age < max_duration
            } else {
                false
            }
        });
    }

    /// Given a message by a user, find the corresponding bot response, if one exists and is cached.
    pub fn find_bot_response(
        &mut self,
        user_msg_id: serenity::MessageId,
    ) -> Option<&mut serenity::Message> {
        let (_, bot_response) = self
            .cache
            .iter_mut()
            .find(|(user_msg, _)| user_msg.id == user_msg_id)?;
        Some(bot_response)
    }

    /// Notify the [`EditTracker`] that the given user message should be associated with the given
    /// bot response.
    fn register_response(&mut self, user_msg: serenity::Message, bot_response: serenity::Message) {
        self.cache.push((user_msg, bot_response));
    }
}

/// Prefix-specific reply function. For more details, see [`crate::send_reply`].
pub async fn send_prefix_reply<'a, U, E>(
    ctx: crate::prefix::PrefixContext<'_, U, E>,
    builder: impl for<'b> FnOnce(&'b mut crate::CreateReply<'a>) -> &'b mut crate::CreateReply<'a>,
) -> Result<Box<serenity::Message>, serenity::Error> {
    let mut reply = crate::CreateReply::default();
    builder(&mut reply);
    let crate::CreateReply {
        content,
        embed,
        attachments,
        components,
        ephemeral: _,
    } = reply;

    let lock_edit_tracker = || {
        // If we definitely don't need to track this command invocation, stop
        let execute_untracked_edits = ctx
            .framework
            .options()
            .prefix_options
            .execute_untracked_edits;
        if !(ctx.command.track_edits || execute_untracked_edits) {
            return None;
        }

        ctx.framework
            .options()
            .prefix_options
            .edit_tracker
            .as_ref()
            .map(|t| t.write().unwrap())
    };

    let existing_response = lock_edit_tracker()
        .as_mut()
        .and_then(|t| t.find_bot_response(ctx.msg.id))
        .cloned();

    Ok(Box::new(if let Some(mut response) = existing_response {
        response
            .edit(ctx.discord, |f| {
                // Empty string resets content (happens when user replaces text with embed)
                f.content(content.as_deref().unwrap_or(""));

                match embed {
                    Some(embed) => f.set_embed(embed),
                    None => f.set_embeds(Vec::new()),
                };

                f.0.insert("attachments", serenity::json::json! { [] }); // reset attachments
                for attachment in attachments {
                    f.attachment(attachment);
                }

                // When components is None, this will still be run to reset the message components
                f.components(|f| {
                    if let Some(components) = components {
                        *f = components;
                    }
                    f
                });

                f
            })
            .await?;

        // If the entry still exists after the await, update it to the new contents
        if let Some(response_entry) = lock_edit_tracker()
            .as_mut()
            .and_then(|t| t.find_bot_response(ctx.msg.id))
        {
            *response_entry = response.clone();
        }

        response
    } else {
        let new_response = ctx
            .msg
            .channel_id
            .send_message(ctx.discord, |m| {
                if let Some(content) = content {
                    m.content(content);
                }
                if let Some(embed) = embed {
                    m.set_embed(embed);
                }
                if let Some(allowed_mentions) = &ctx.framework.options().allowed_mentions {
                    m.allowed_mentions(|m| {
                        *m = allowed_mentions.clone();
                        m
                    });
                }
                if let Some(components) = components {
                    m.components(|c| {
                        c.0 = components.0;
                        c
                    });
                }

                for attachment in attachments {
                    m.add_file(attachment);
                }
                m
            })
            .await?;
        if let Some(track_edits) = &mut lock_edit_tracker() {
            track_edits.register_response(ctx.msg.clone(), new_response.clone());
        }

        new_response
    }))
}
