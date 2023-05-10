use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Query information about a Discord profile
#[poise::command(context_menu_command = "User information", slash_command)]
pub async fn user_info(
    ctx: Context<'_>,
    #[description = "Discord profile to query information about"] user: serenity::User,
) -> Result<(), Error> {
    let response = format!(
        "**Name**: {}\n**Created**: {}",
        user.name,
        user.created_at()
    );

    ctx.say(response).await?;
    Ok(())
}

/// Echo content of a message
#[poise::command(context_menu_command = "Echo", slash_command)]
pub async fn echo(
    ctx: Context<'_>,
    #[description = "Message to echo (enter a link or ID)"] msg: serenity::Message,
) -> Result<(), Error> {
    ctx.say(&msg.content).await?;
    Ok(())
}
