use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn reply(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply(format!("Hello {}!", ctx.author().name)).await?;
    Ok(())
}
