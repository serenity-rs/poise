use crate::serenity_prelude as serenity;

#[derive(Default)]
pub struct CreateReply<'a> {
    pub content: Option<String>,
    pub embed: Option<serenity::CreateEmbed>,
    pub attachments: Vec<serenity::AttachmentType<'a>>,
    pub ephemeral: bool,
}

impl<'a> CreateReply<'a> {
    /// Set the content of the message.
    pub fn content(&mut self, content: String) -> &mut Self {
        self.content = Some(content);
        self
    }

    /// Set an embed for the message.
    ///
    /// Any previously set embed will be overwritten.
    pub fn embed(
        &mut self,
        f: impl FnOnce(&mut serenity::CreateEmbed) -> &mut serenity::CreateEmbed,
    ) -> &mut Self {
        let mut embed = serenity::CreateEmbed::default();
        f(&mut embed);
        self.embed = Some(embed);
        self
    }

    /// Add an attachment.
    ///
    /// This will not have an effect in a slash command's initial response!
    pub fn attachment(&mut self, attachment: serenity::AttachmentType<'a>) -> &mut Self {
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
}

pub async fn send_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: impl for<'a, 'b> FnOnce(&'a mut CreateReply<'b>) -> &'a mut CreateReply<'b>,
) -> Result<(), serenity::Error> {
    match ctx {
        crate::Context::Prefix(ctx) => crate::send_prefix_reply(ctx, builder).await,
        crate::Context::Slash(ctx) => crate::send_slash_reply(ctx, builder).await,
    }
}

pub async fn say_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: String,
) -> Result<(), serenity::Error> {
    send_reply(ctx, |m| m.content(text)).await
}
