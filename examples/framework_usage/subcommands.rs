use crate::{Context, Error};

// Subcommands are defined independently of their parent. The parent-child relation is set when
// adding commands into the framework, using `.subcommand()` on the command builder callback (See
// main.rs)

/// A command with two subcommands: `child1` and `child2`
///
/// This command can only be called as a prefix command with `~parent`. Discord doesn't permit
/// invoking the root command of a slash command.
#[poise::command(prefix_command, slash_command)]
pub async fn parent(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}

/// A subcommand of `parent`
#[poise::command(prefix_command, slash_command)]
pub async fn child1(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked `parent child1`!").await?;
    Ok(())
}

/// Another subcommand of `parent`
#[poise::command(prefix_command, slash_command)]
pub async fn child2(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked `parent child2`!").await?;
    Ok(())
}
