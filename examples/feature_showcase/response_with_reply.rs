use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn reply(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(|b| {
        b.content(format!("Hello {}!", ctx.author().name))
            .reply(true)
    })
    .await?;
    Ok(())
}
