use crate::serenity_prelude as serenity;

#[derive(Default)]
pub struct CreateReply {
    pub content: Option<String>,
    pub embed: Option<serenity::CreateEmbed>,
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
