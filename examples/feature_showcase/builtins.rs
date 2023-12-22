use crate::{Context, Error};

#[cfg(feature = "cache")]
#[poise::command(slash_command, prefix_command)]
pub async fn servers(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::servers(ctx).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let configuration = poise::builtins::HelpConfiguration {
        // [configure aspects about the help message here]
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn pretty_help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let configuration = poise::builtins::PrettyHelpConfiguration {
        // [configure aspects about the help message here]
        ..Default::default()
    };
    poise::builtins::pretty_help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}
