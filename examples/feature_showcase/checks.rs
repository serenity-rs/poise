/**
Poise supports several pre-command checks (sorted by order of execution):
- owners_only
- required_permissions
- required_bot_permissions
- global check function
- command-specific check function
- cooldowns
*/
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}

/// A moderator-only command, using required_permissions
#[poise::command(
    prefix_command,
    slash_command,
    // Multiple permissions can be OR-ed together with `|` to make them all required
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
)]
pub async fn modonly(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You are a mod because you were able to invoke this command")
        .await?;
    Ok(())
}

/// Deletes the given message
#[poise::command(
    prefix_command,
    slash_command,
    required_bot_permissions = "MANAGE_MESSAGES"
)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "Message to be deleted"] msg: serenity::Message,
) -> Result<(), Error> {
    msg.delete(ctx).await?;
    Ok(())
}

/// Returns true if username is Ferris
async fn is_ferris(ctx: Context<'_>) -> Result<bool, Error> {
    let nickname = match ctx.guild_id() {
        Some(guild_id) => ctx.author().nick_in(ctx, guild_id).await,
        None => None,
    };
    let name = nickname.as_ref().unwrap_or(&ctx.author().name);

    Ok(name.eq_ignore_ascii_case("ferris"))
}

/// Always passes the check and prints a line on the console
async fn noisy_check(_ctx: Context<'_>) -> Result<bool, Error> {
    println!("Noisy check has been called!");
    Ok(true)
}

/// Crab party... only for "Ferris"!
#[poise::command(
    prefix_command,
    slash_command,
    check = "is_ferris",
    // You can write `check = ...` multiple times to add multiple checks
    check = "noisy_check",
)]
pub async fn ferrisparty(ctx: Context<'_>) -> Result<(), Error> {
    let response = "```\n".to_owned()
        + &r"    _~^~^~_
\) /  o o  \ (/
  '_   ¬   _'
  | '-----' |
"
        .repeat(3)
        + "```";
    ctx.say(response).await?;
    Ok(())
}

/// Add two numbers
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    // All cooldowns in seconds
    global_cooldown = 1,
    user_cooldown = 5,
    guild_cooldown = 2,
    channel_cooldown = 2,
    member_cooldown = 3,
)]
pub async fn cooldowns(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You successfully called the command").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn minmax(
    ctx: Context<'_>,
    #[min = -15]
    #[max = 28.765]
    value: f32,
) -> Result<(), Error> {
    ctx.say(format!("You submitted number {}", value)).await?;
    Ok(())
}

/// Get the guild name (guild-only)
#[poise::command(prefix_command, slash_command, guild_only)]
pub async fn get_guild_name(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(
        "The name of this guild is: {}",
        ctx.partial_guild().await.unwrap().name
    ))
    .await?;

    Ok(())
}

/// A dm-only command
#[poise::command(prefix_command, slash_command, dm_only)]
pub async fn only_in_dms(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This is a dm channel").await?;

    Ok(())
}

/// Only runs on NSFW channels
#[poise::command(prefix_command, slash_command, nsfw_only)]
pub async fn lennyface(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("( ͡° ͜ʖ ͡°)").await?;

    Ok(())
}

/// Utilizes the permissions v2 `default_member_permissions` field
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn permissions_v2(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Whoop! You're authorized!").await?;

    Ok(())
}
