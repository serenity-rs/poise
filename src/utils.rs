use crate::serenity;

pub fn update_message(message: &mut serenity::Message, update: serenity::MessageUpdateEvent) {
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
