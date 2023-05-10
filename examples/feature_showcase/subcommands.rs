use crate::{Context, Error};

/// A command with two subcommands: `child1` and `child2`
///
/// Running this function directly, without any subcommand, is only supported in prefix commands.
/// Discord doesn't permit invoking the root command of a slash command if it has subcommands.
#[poise::command(prefix_command, slash_command, subcommands("child1", "child2"))]
pub async fn parent(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}

/// A subcommand of `parent`
#[poise::command(prefix_command, slash_command)]
pub async fn child1(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the first child command!").await?;
    Ok(())
}

/// Another subcommand of `parent`
#[poise::command(prefix_command, slash_command)]
pub async fn child2(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the second child command!").await?;
    Ok(())
}
