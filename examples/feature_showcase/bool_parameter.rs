use crate::{Context, Error};

/// Tests poise's bool parameter
///
/// In prefix commands, many affirmative words and their opposites are supported
#[poise::command(slash_command, prefix_command)]
pub async fn oracle(
    ctx: Context<'_>,
    #[description = "Take a decision"] b: bool,
) -> Result<(), Error> {
    if b {
        ctx.say("You seem to be an optimistic kind of person...")
            .await?;
    } else {
        ctx.say("You seem to be a pessimistic kind of person...")
            .await?;
    }
    Ok(())
}
