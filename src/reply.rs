use crate::serenity_prelude as serenity;

#[derive(Default)]
pub struct CreateReply<'a> {
    pub content: Option<String>,
    pub embed: Option<serenity::CreateEmbed>,
    pub attachments: Vec<serenity::AttachmentType<'a>>,
    pub ephemeral: bool,
    pub components: Option<serenity::CreateComponents>,
}

impl<'a> CreateReply<'a> {
    /// Set the content of the message.
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = Some(content.into());
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

    /// Set an embed for the message.
    ///
    /// Any previously set embed will be overwritten.
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

/// Returned from [`send_reply`] to retrieve the sent message object.
///
/// For prefix commands, you can retrieve the sent message directly. For slash commands, Discord
/// requires a network request.
pub enum ReplyHandle<'a> {
    Prefix(serenity::Message),
    Slash {
        http: &'a serenity::Http,
        interaction: &'a serenity::ApplicationCommandInteraction,
    },
}

impl ReplyHandle<'_> {
    pub async fn message(self) -> Result<serenity::Message, serenity::Error> {
        match self {
            Self::Prefix(msg) => Ok(msg),
            Self::Slash { http, interaction } => interaction.get_interaction_response(http).await,
        }
    }
}

pub async fn send_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    builder: impl for<'a, 'b> FnOnce(&'a mut CreateReply<'b>) -> &'a mut CreateReply<'b>,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    Ok(match ctx {
        crate::Context::Prefix(ctx) => {
            ReplyHandle::Prefix(crate::send_prefix_reply(ctx, builder).await?)
        }
        crate::Context::Slash(ctx) => {
            crate::send_slash_reply(ctx, builder).await?;
            ReplyHandle::Slash {
                interaction: ctx.interaction,
                http: &ctx.discord.http,
            }
        }
    })
}

pub async fn say_reply<U, E>(
    ctx: crate::Context<'_, U, E>,
    text: impl Into<String>,
) -> Result<ReplyHandle<'_>, serenity::Error> {
    send_reply(ctx, |m| m.content(text.into())).await
}
