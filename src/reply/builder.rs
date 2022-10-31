//! The builder to create a new reply

use crate::serenity_prelude as serenity;

/// Message builder that abstracts over prefix and application command responses
#[derive(Default, Clone)]
pub struct CreateReply<'att> {
    /// Message content.
    pub content: Option<String>,
    /// Embeds, if present.
    pub embeds: Vec<serenity::CreateEmbed>,
    /// Message attachments.
    pub attachments: Vec<serenity::AttachmentType<'att>>,
    /// Whether the message is ephemeral (only has an effect in application commands)
    pub ephemeral: bool,
    /// Message components, that is, buttons and select menus.
    pub components: Option<serenity::CreateComponents>,
    /// The allowed mentions for the message.
    pub allowed_mentions: Option<serenity::CreateAllowedMentions>,
    /// The reference message this message is a reply to.
    pub reference_message: Option<serenity::MessageReference>,
}

impl<'att> CreateReply<'att> {
    /// Set the content of the message.
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
        self
    }

    /// Adds an embed to the message.
    ///
    /// Existing embeds are kept.
    pub fn embed(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateEmbed) -> &mut serenity::CreateEmbed,
    ) -> &mut Self {
        let mut embed = serenity::CreateEmbed::default();
        f(&mut embed);
        self.embeds.push(embed);
        self
    }

    /// Set components (buttons and select menus) for this message.
    ///
    /// Any previously set components will be overwritten.
    pub fn components(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateComponents) -> &mut serenity::CreateComponents,
    ) -> &mut Self {
        let mut components = serenity::CreateComponents::default();
        f(&mut components);
        self.components = Some(components);
        self
    }

    /// Add an attachment.
    ///
    /// This will not have an effect in a slash command's initial response!
    pub fn attachment(&mut self, attachment: serenity::AttachmentType<'att>) -> &mut Self {
        self.attachments.push(attachment);
        self
    }

    /// Toggles whether the message is an ephemeral response (only invoking user can see it).
    ///
    /// This only has an effect in slash commands!
    pub fn ephemeral(&mut self, ephemeral: bool) -> &mut Self {
        self.ephemeral = ephemeral;
        self
    }

    /// Set the allowed mentions for the message.
    ///
    /// See [`serenity::CreateAllowedMentions`] for more information.
    pub fn allowed_mentions(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateAllowedMentions) -> &mut serenity::CreateAllowedMentions,
    ) -> &mut Self {
        let mut allowed_mentions = serenity::CreateAllowedMentions::default();
        f(&mut allowed_mentions);
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Set the reference message this message is a reply to.
    pub fn reference_message(
        &mut self,
        reference: impl Into<serenity::MessageReference>,
    ) -> &mut Self {
        self.reference_message = Some(reference.into());
        self
    }
}

/// Methods to create a message builder from any type from this [`CreateReply`]. Used by poise
/// internally to actually send a response to Discord
impl<'att> CreateReply<'att> {
    /// Serialize this response builder to a [`serenity::CreateInteractionResponseData`]
    pub fn to_slash_initial_response(self, f: &mut serenity::CreateInteractionResponseData<'att>) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            reference_message: _, // can't reply to a message in interactions
        } = self;

        if let Some(content) = content {
            f.content(content);
        }
        f.set_embeds(embeds);
        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|f| {
                *f = allowed_mentions.clone();
                f
            });
        }
        if let Some(components) = components {
            f.components(|f| {
                f.0 = components.0;
                f
            });
        }
        f.ephemeral(ephemeral);
        f.add_files(attachments);
    }

    /// Serialize this response builder to a [`serenity::CreateInteractionResponseFollowup`]
    pub fn to_slash_followup_response(
        self,
        f: &mut serenity::CreateInteractionResponseFollowup<'att>,
    ) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            reference_message: _,
        } = self;

        if let Some(content) = content {
            f.content(content);
        }
        f.set_embeds(embeds);
        if let Some(components) = components {
            f.components(|c| {
                c.0 = components.0;
                c
            });
        }
        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|f| {
                *f = allowed_mentions.clone();
                f
            });
        }
        f.ephemeral(ephemeral);
        f.add_files(attachments);
    }

    /// Serialize this response builder to a [`serenity::EditInteractionResponse`]
    pub fn to_slash_initial_response_edit(self, f: &mut serenity::EditInteractionResponse) {
        let crate::CreateReply {
            content,
            embeds,
            attachments: _, // no support for attachment edits in serenity yet
            components,
            ephemeral: _, // can't edit ephemerality in retrospect
            allowed_mentions,
            reference_message: _,
        } = self;

        if let Some(content) = content {
            f.content(content);
        }
        f.set_embeds(embeds);
        if let Some(components) = components {
            f.components(|c| {
                c.0 = components.0;
                c
            });
        }
        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|f| {
                *f = allowed_mentions.clone();
                f
            });
        }
    }

    /// Serialize this response builder to a [`serenity::EditMessage`]
    pub fn to_prefix_edit(self, f: &mut serenity::EditMessage<'att>) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            reference_message: _, // can't edit reference message afterwards
        } = self;

        if let Some(content) = content {
            f.content(content);
        }
        f.add_embeds(embeds);
        for attachment in attachments {
            f.attachment(attachment);
        }

        if let Some(allowed_mentions) = allowed_mentions {
            f.allowed_mentions(|b| {
                *b = allowed_mentions;
                b
            });
        }

        if let Some(components) = components {
            f.components(|f| {
                *f = components;
                f
            });
        }
    }

    /// Serialize this response builder to a [`serenity::CreateMessage`]
    pub fn to_prefix(self, m: &mut serenity::CreateMessage<'att>) {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            reference_message,
        } = self;

        if let Some(content) = content {
            m.content(content);
        }
        m.set_embeds(embeds);
        if let Some(allowed_mentions) = allowed_mentions {
            m.allowed_mentions(|m| {
                *m = allowed_mentions;
                m
            });
        }
        if let Some(components) = components {
            m.components(|c| {
                c.0 = components.0;
                c
            });
        }
        if let Some(reference_message) = reference_message {
            m.reference_message(reference_message);
        }

        for attachment in attachments {
            m.add_file(attachment);
        }
    }
}
