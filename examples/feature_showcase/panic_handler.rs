use crate::{Context, Error};

/// This command panics when dividing by zero
///
/// This will be caught by poise's panic handler
#[poise::command(slash_command, prefix_command)]
pub async fn div(ctx: Context<'_>, a: i32, b: i32) -> Result<(), Error> {
    ctx.say((a / b).to_string()).await?;
    Ok(())
}
