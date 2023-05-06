use crate::{Context, Error};
#[allow(unused_imports)]
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command)]
pub async fn paginate(ctx: Context<'_>) -> Result<(), Error> {
    let pages = &[
        "Content of first page",
        "Content of second page",
        "Content of third page",
        "Content of fourth page",
    ];

    poise::samples::paginate(ctx, pages).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn div(ctx: Context<'_>, a: i32, b: i32) -> Result<(), Error> {
    ctx.say((a / b).to_string()).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn stringlen(
    ctx: Context<'_>,
    #[min_length = 3]
    #[max_length = 5]
    s: String,
) -> Result<(), Error> {
    ctx.say(format!("you wrote: {}", s)).await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn role(ctx: Context<'_>, role: serenity::Role) -> Result<(), Error> {
    ctx.say(format!("yes this is indeed a role: {:?}", role))
        .await?;
    Ok(())
}
