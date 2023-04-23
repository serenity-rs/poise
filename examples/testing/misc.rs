use crate::{Context, Error};

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
