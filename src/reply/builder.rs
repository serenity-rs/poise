//! The builder to create a new reply

use crate::serenity_prelude as serenity;

/// Message builder that abstracts over prefix and application command responses
#[derive(Default, Clone)]
#[allow(clippy::missing_docs_in_private_items)] // docs on setters
pub struct CreateReply {
    content: Option<String>,
    embeds: Option<Vec<serenity::CreateEmbed>>,
    attachments: Vec<serenity::CreateAttachment>,
    pub(crate) ephemeral: Option<bool>,
    components: Option<Vec<serenity::CreateActionRow>>,
    pub(crate) allowed_mentions: Option<serenity::CreateAllowedMentions>,
    reply: bool,
}

impl CreateReply {
    /// Creates a blank CreateReply. Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content of the message.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Adds an embed to the message.
    ///
    /// Existing embeds on this are kept.
    /// When editing a message, this will overwrite previously sent embeds.
    pub fn embed(mut self, embed: serenity::CreateEmbed) -> Self {
        self.embeds.get_or_insert_with(|| Default::default()).push(embed);
        self
    }

    /// Set embeds for the message.
    ///
    /// Any previously set embeds will be overwritten.
    pub fn embeds(mut self, embeds: Vec<serenity::CreateEmbed>) -> Self {
        self.embeds = Some(embeds);
        self
    }

    /// Set components (buttons and select menus) for this message.
    ///
    /// Any previously set components will be overwritten.
    pub fn components(mut self, components: Vec<serenity::CreateActionRow>) -> Self {
        self.components = Some(components);
        self
    }

    /// Add an attachment.
    ///
    /// This will not have an effect in a slash command's initial response!
    pub fn attachment(mut self, attachment: serenity::CreateAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Toggles whether the message is an ephemeral response (only invoking user can see it).
    ///
    /// This only has an effect in slash commands!
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = Some(ephemeral);
        self
    }

    /// Set the allowed mentions for the message.
    ///
    /// See [`serenity::CreateAllowedMentions`] for more information.
    pub fn allowed_mentions(mut self, allowed_mentions: serenity::CreateAllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Makes this message an inline reply to another message like [`serenity::Message::reply`]
    /// (prefix-only, because slash commands are always inline replies anyways).
    ///
    /// To disable the ping, set [`Self::allowed_mentions`] with
    /// [`serenity::CreateAllowedMentions::replied_user`] set to false.
    pub fn reply(mut self, reply: bool) -> Self {
        self.reply = reply;
        self
    }
}

/// Methods to create a message builder from any type from this [`CreateReply`]. Used by poise
/// internally to actually send a response to Discord
impl CreateReply {
    /// Serialize this response builder to a [`serenity::CreateInteractionResponseMessage`]
    pub fn to_slash_initial_response(
        self,
        mut builder: serenity::CreateInteractionResponseMessage,
    ) -> serenity::CreateInteractionResponseMessage {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            reply: _, // can't reply to a message in interactions
        } = self;

        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }
        if let Some(embeds) = embeds {
            builder = builder.embeds(embeds);
        }
        if let Some(ephemeral) = ephemeral {
            builder = builder.ephemeral(ephemeral);
        }

        builder.add_files(attachments)
    }

    /// Serialize this response builder to a [`serenity::CreateInteractionResponseFollowup`]
    pub fn to_slash_followup_response(
        self,
        mut builder: serenity::CreateInteractionResponseFollowup,
    ) -> serenity::CreateInteractionResponseFollowup {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            reply: _,
        } = self;

        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(components) = components {
            builder = builder.components(components)
        }
        if let Some(embeds) = embeds {
            builder = builder.embeds(embeds);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(ephemeral) = ephemeral {
            builder = builder.ephemeral(ephemeral);
        }

        builder.add_files(attachments)
    }

    /// Serialize this response builder to a [`serenity::EditInteractionResponse`]
    pub fn to_slash_initial_response_edit(
        self,
        mut builder: serenity::EditInteractionResponse,
    ) -> serenity::EditInteractionResponse {
        let crate::CreateReply {
            content,
            embeds,
            attachments: _, // no support for attachment edits in serenity yet
            components,
            ephemeral: _, // can't edit ephemerality in retrospect
            allowed_mentions,
            reply: _,
        } = self;

        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }
        if let Some(embeds) = embeds {
            builder = builder.embeds(embeds);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }

        builder
    }

    /// Serialize this response builder to a [`serenity::EditMessage`]
    pub fn to_prefix_edit(self, mut builder: serenity::EditMessage) -> serenity::EditMessage {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            reply: _, // can't edit reference message afterwards
        } = self;

        let mut attachments_builder = serenity::EditAttachments::new();
        for attachment in attachments {
            attachments_builder = attachments_builder.add(attachment);
        }

        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }
        if let Some(embeds) = embeds {
            builder = builder.embeds(embeds);
        }

        builder.attachments(attachments_builder)
    }

    /// Serialize this response builder to a [`serenity::CreateMessage`]
    pub fn to_prefix(
        self,
        invocation_message: serenity::MessageReference,
    ) -> serenity::CreateMessage {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            reply,
        } = self;

        let mut builder = serenity::CreateMessage::new();
        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }
        if let Some(embeds) = embeds {
            builder = builder.embeds(embeds)
        }
        if reply {
            builder = builder.reference_message(invocation_message);
        }

        for attachment in attachments {
            builder = builder.add_file(attachment);
        }

        builder
    }
}
