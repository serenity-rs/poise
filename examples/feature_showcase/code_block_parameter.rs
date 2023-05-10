use crate::{Context, Error};

#[poise::command(prefix_command)]
pub async fn code(
    ctx: Context<'_>,
    args: poise::KeyValueArgs,
    code: poise::CodeBlock,
) -> Result<(), Error> {
    ctx.say(format!("Key value args: {:?}\nCode: {}", args, code))
        .await?;
    Ok(())
}
