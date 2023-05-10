use crate::{Context, Error};

/// A command with two subcommands: `child1` and `child2`
///
/// Running this function directly, without any subcommand, is only supported in prefix commands.
/// Discord doesn't permit invoking the root command of a slash command if it has subcommands.
/// This command can be invoked only with `parent child1` and `parent child2`, due to `subcommand_required` parameter.
/// If you want to allow `parent` to be invoked without subcommand, remove `subcommand_required` parameter
#[poise::command(prefix_command, slash_command, subcommands("child1", "child2"), subcommand_required)]
// Omit 'ctx' parameter here. It is not needed, because this function will never be called.
// TODO: Add a way to remove 'ctx' parameter, when `subcommand_required` is set
pub async fn parent(_: Context<'_>) -> Result<(), Error> {
    // This will never be called, because `subcommand_required` parameter is set
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
