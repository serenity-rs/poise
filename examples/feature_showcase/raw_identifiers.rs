use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn r#move(ctx: Context<'_>, r#loop: String, r#fn: String) -> Result<(), Error> {
    ctx.say(format!("called with loop={} and fn={}", r#loop, r#fn))
        .await?;
    Ok(())
}
