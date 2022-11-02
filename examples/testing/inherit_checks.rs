use crate::{Context, Error};

async fn child2_check(_ctx: Context<'_>) -> Result<bool, Error> {
    println!("Child2 check executed!");
    Ok(true)
}
async fn child1_check(_ctx: Context<'_>) -> Result<bool, Error> {
    println!("Child1 check executed!");
    Ok(true)
}
async fn parent_check(_ctx: Context<'_>) -> Result<bool, Error> {
    println!("Parent check executed!");
    Ok(true)
}

#[poise::command(slash_command, prefix_command, check = "child2_check")]
async fn child2(ctx: Context<'_>, _b: bool, _s: String, _i: u32) -> Result<(), Error> {
    ctx.say(ctx.invocation_string()).await?;
    Ok(())
}
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("child2"),
    check = "child1_check"
)]
async fn child1(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("child1"),
    check = "parent_check"
)]
pub async fn parent(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
