use crate::serenity;

pub struct EditTracker {
    max_duration: std::time::Duration,
    cache: Vec<(serenity::Message, serenity::Message)>,
}

impl EditTracker {
    pub fn for_timespan(duration: std::time::Duration) -> parking_lot::RwLock<Self> {
        parking_lot::RwLock::new(Self {
            max_duration: duration,
            cache: Vec::new(),
        })
    }

    /// Returns a reference to the bot response that was just registered
    pub fn register_response(
        &mut self,
        user_msg: serenity::Message,
        bot_response: serenity::Message,
    ) {
        self.cache.push((user_msg, bot_response));
    }

    /// Returns a copy of a newly up-to-date cached message, or a brand new generated message when
    /// not in cache
    pub fn process_message_update(
        &mut self,
        user_msg_update: &serenity::MessageUpdateEvent,
    ) -> serenity::Message {
        match self
            .cache
            .iter_mut()
            .find(|(user_msg, _)| user_msg.id == user_msg_update.id)
        {
            Some((user_msg, _)) => {
                crate::utils::update_message(user_msg, user_msg_update.clone());
                user_msg.clone()
            }
            None => {
                let mut user_msg = serenity::CustomMessage::new().build();
                crate::utils::update_message(&mut user_msg, user_msg_update.clone());
                user_msg
            }
        }
    }

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
}

pub struct CreateReply {
    content: Option<String>,
    embed: Option<serenity::CreateEmbed>,
}

impl CreateReply {
    /// Set the content of the message.
    pub fn content(&mut self, content: String) -> &mut Self {
        self.content = Some(content);
        self
    }

    /// Set an embed for the message.
    pub fn embed<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut serenity::CreateEmbed) -> &mut serenity::CreateEmbed,
    {
        let mut embed = serenity::CreateEmbed::default();
        f(&mut embed);
        self.embed = Some(embed);
        self
    }
}

pub async fn send_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: impl FnOnce(&mut CreateReply) -> &mut CreateReply,
) -> Result<(), serenity::Error> {
    let mut reply = CreateReply {
        content: None,
        embed: None,
    };
    builder(&mut reply);

    let mut track_edits = ctx
        .framework
        .options
        .edit_tracker
        .as_ref()
        .map(|t| t.write());

    match track_edits
        .as_mut()
        .and_then(|t| t.find_bot_response(ctx.msg.id))
    {
        Some(existing_response) => {
            existing_response
                .edit(ctx.discord, |m| {
                    if let Some(content) = reply.content {
                        m.content(content);
                    }
                    if let Some(embed) = reply.embed {
                        m.embed(|e| {
                            *e = embed;
                            e
                        });
                    }
                    m
                })
                .await?;
        }
        None => {
            let new_response = ctx
                .msg
                .channel_id
                .send_message(ctx.discord, |m| {
                    if let Some(content) = reply.content {
                        m.content(content);
                    }
                    if let Some(embed) = reply.embed {
                        m.embed(|e| {
                            *e = embed;
                            e
                        });
                    }
                    m
                })
                .await?;
            if let Some(track_edits) = &mut track_edits {
                track_edits.register_response(ctx.msg.clone(), new_response);
            }
        }
    }
    Ok(())
}

pub async fn say_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: String,
) -> Result<(), serenity::Error> {
    send_reply(ctx, |m| m.content(text)).await
}
