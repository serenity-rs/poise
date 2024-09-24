use crate::{Context, Error};
use poise::serenity_prelude as serenity;

// `install_context` determines how the bot has to be installed for a command to be available.
// `interaction_context` determines where a command can be used.

/// Available everywhere
#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn everywhere(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This command is available everywhere!").await?;
    Ok(())
}

// also works with `context_menu_command`
/// Available everywhere
#[poise::command(
    context_menu_command = "Everywhere",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn everywhere_context(ctx: Context<'_>, msg: serenity::Message) -> Result<(), Error> {
    msg.reply(ctx, "This context menu is available everywhere!")
        .await?;
    Ok(())
}

/// Available with a user install only
#[poise::command(
    slash_command,
    install_context = "User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn user_install(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This command is available only with a user install!")
        .await?;
    Ok(())
}

/// Not available in guilds
#[poise::command(
    slash_command,
    install_context = "User",
    interaction_context = "BotDm|PrivateChannel"
)]
pub async fn not_in_guilds(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This command is not available in guilds!").await?;
    Ok(())
}

/// User install only in guilds
#[poise::command(slash_command, install_context = "User", interaction_context = "Guild")]
pub async fn user_install_guild(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This command is available in guilds only with a user install!")
        .await?;
    Ok(())
}
