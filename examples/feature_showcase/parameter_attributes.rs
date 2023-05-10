use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Adds multiple numbers
///
/// Demonstrates `#[min]` and `#[max]`
#[poise::command(prefix_command, slash_command)]
pub async fn addmultiple(
    ctx: Context<'_>,
    #[description = "An operand"] a: i8,
    #[description = "An operand"] b: u64,
    #[description = "An operand"]
    #[min = 1234567890123456_i64]
    #[max = 1234567890987654_i64]
    c: i64,
) -> Result<(), Error> {
    ctx.say(format!("Result: {}", a as i128 + b as i128 + c as i128))
        .await?;

    Ok(())
}

/// Demonstrates `#[channel_types]`
#[poise::command(slash_command)]
pub async fn voiceinfo(
    ctx: Context<'_>,
    #[description = "Information about a server voice channel"]
    #[channel_types("Voice")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let response = format!(
        "\
**Name**: {}
**Bitrate**: {}
**User limit**: {}
**RTC region**: {}
**Video quality mode**: {:?}",
        channel.name,
        channel.bitrate.unwrap_or_default(),
        channel.user_limit.unwrap_or_default(),
        channel.rtc_region.unwrap_or_default(),
        channel
            .video_quality_mode
            .unwrap_or(serenity::VideoQualityMode::Unknown)
    );

    ctx.say(response).await?;
    Ok(())
}

/// Echoes the string you give it
///
/// Demonstrates `#[rest]`
#[poise::command(prefix_command, slash_command)]
pub async fn say(
    ctx: Context<'_>,
    #[rest]
    #[description = "Text to say"]
    msg: String,
) -> Result<(), Error> {
    ctx.say(msg).await?;
    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum PunishType {
    Ban,
    Kick,
    Mute,
}

/// Punishment command for testing the rename macro
#[poise::command(slash_command)]
pub async fn punish(
    ctx: Context<'_>,
    #[description = "Punishment type"]
    #[rename = "type"]
    punish_type: PunishType,
    #[description = "User to execute the punishment on"] user: serenity::User,
) -> Result<(), Error> {
    let text = match punish_type {
        PunishType::Ban => format!("{} has been banned!", user.name),
        PunishType::Kick => format!("{} has been kicked!", user.name),
        PunishType::Mute => format!("{} has been muted!", user.name),
    };
    ctx.say(text).await?;

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
