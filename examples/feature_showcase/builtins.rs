use crate::{Context, Error};

#[cfg(feature = "cache")]
#[poise::command(slash_command, prefix_command)]
pub async fn servers(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::servers(ctx).await?;
    Ok(())
}
