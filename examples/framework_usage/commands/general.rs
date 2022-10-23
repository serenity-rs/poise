use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, CreateActionRow};
use std::fmt::Write as _;

/// Vote for something
///
/// Enter `~vote pumpkin` to vote for pumpkins
#[poise::command(prefix_command, slash_command)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "What to vote for"] choice: String,
) -> Result<(), Error> {
    let num_votes = {
        let mut hash_map = ctx.data().votes.lock().unwrap();
        let num_votes = hash_map.entry(choice.clone()).or_default();
        *num_votes += 1;
        *num_votes
    };

    let response = format!(
        "Successfully voted for {0}. {0} now has {1} votes!",
        choice, num_votes
    );
    ctx.say(response).await?;
    Ok(())
}

/// Retrieve number of votes
///
/// Retrieve the number of votes either in general, or for a specific choice:
/// ```
/// ~getvotes
/// ~getvotes pumpkin
/// ```
#[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
pub async fn getvotes(
    ctx: Context<'_>,
    #[description = "Choice to retrieve votes for"] choice: Option<String>,
) -> Result<(), Error> {
    if let Some(choice) = choice {
        let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
        let response = match num_votes {
            0 => format!("Nobody has voted for {} yet", choice),
            _ => format!("{} people have voted for {}", num_votes, choice),
        };
        ctx.say(response).await?;
    } else {
        let mut response = String::new();
        for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
            let _ = writeln!(response, "{}: {} votes", choice, num_votes);
        }

        if response.is_empty() {
            response += "Nobody has voted for anything yet :(";
        }

        ctx.say(response).await?;
    };

    Ok(())
}

/// Adds multiple numbers
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

#[derive(Debug, poise::ChoiceParameter)]
pub enum MyStringChoice {
    #[name = "The first choice"]
    A,
    #[name = "The second choice"]
    #[name = "A single choice can have multiple names"]
    B,
    // If no name is given, the variant name is used
    C,
}

/// Dummy command to test slash command choice parameters
#[poise::command(prefix_command, slash_command)]
pub async fn choice(
    ctx: Context<'_>,
    #[description = "The choice you want to choose"] choice: MyStringChoice,
) -> Result<(), Error> {
    ctx.say(format!("You entered {:?}", choice)).await?;
    Ok(())
}

/// Boop the bot!
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn boop(ctx: Context<'_>) -> Result<(), Error> {
    let uuid_boop = ctx.id();

    ctx.send(
        poise::CreateReply::default()
            .content("I want some boops!")
            .components(vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(
                    "Boop me!",
                    serenity::ButtonStyle::Primary,
                    uuid_boop.to_string(),
                ),
            ])]),
    )
    .await?;

    let mut boop_count = 0;
    while let Some(mci) = serenity::ComponentInteractionCollectorBuilder::new(&ctx.discord().shard)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(std::sync::Arc::new(move |mci| {
            mci.data.custom_id == uuid_boop.to_string()
        }))
        .collect_single()
        .await
    {
        boop_count += 1;

        let mut msg = mci.message.clone();
        msg.edit(
            ctx.discord(),
            serenity::EditMessage::default().content(format!("Boop count: {}", boop_count)),
        )
        .await?;

        mci.create_interaction_response(
            ctx.discord(),
            serenity::CreateInteractionResponse::Acknowledge,
        )
        .await?;
    }

    Ok(())
}

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
            .unwrap_or(serenity::VideoQualityMode::Unknown(0))
    );

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, reuse_response)]
pub async fn test_reuse_response(ctx: Context<'_>) -> Result<(), Error> {
    let image_url = "https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png";

    ctx.send(
        poise::CreateReply::default()
            .content("message 1")
            .embed(
                serenity::CreateEmbed::default()
                    .description("embed 1")
                    .image(image_url),
            )
            .components(vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("button 1", serenity::ButtonStyle::Primary, "1"),
            ])]),
    )
    .await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let image_url = "https://raw.githubusercontent.com/serenity-rs/serenity/current/examples/e09_create_message_builder/ferris_eyes.png";

    ctx.send(
        poise::CreateReply::default()
            .content("message 2")
            .embed(
                serenity::CreateEmbed::default()
                    .description("embed 2")
                    .image(image_url),
            )
            .components(vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("button 2", serenity::ButtonStyle::Primary, "2"),
            ])]),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn oracle(
    ctx: Context<'_>,
    #[description = "Take a decision"] b: bool,
) -> Result<(), Error> {
    if b {
        ctx.say("You seem to be an optimistic kind of person...")
            .await?;
    } else {
        ctx.say("You seem to be a pessimistic kind of person...")
            .await?;
    }
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn code(
    ctx: Context<'_>,
    args: poise::KeyValueArgs,
    code: poise::CodeBlock,
) -> Result<(), Error> {
    ctx.say(format!("Key value args: {:?}\nCode: {}", args, code))
        .await?;
    Ok(())
}

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

/// View the difference between two file sizes
#[poise::command(prefix_command, slash_command)]
pub async fn file_details(
    ctx: Context<'_>,
    #[description = "File to examine"] file: serenity::Attachment,
    #[description = "Second file to examine"] file_2: Option<serenity::Attachment>,
) -> Result<(), Error> {
    ctx.say(format!(
        "First file name: **{}**. File size difference: **{}** bytes",
        file.filename,
        file.size - file_2.map_or(0, |f| f.size)
    ))
    .await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn totalsize(
    ctx: Context<'_>,
    #[description = "File to rename"] files: Vec<serenity::Attachment>,
) -> Result<(), Error> {
    let total = files.iter().map(|f| f.size).sum::<u64>();

    ctx.say(format!(
        "Total file size: `{}B`. Average size: `{}B`",
        total,
        total.checked_div(files.len() as _).unwrap_or(0)
    ))
    .await?;

    Ok(())
}

#[derive(Debug, poise::Modal)]
#[allow(dead_code)] // fields only used for Debug print
struct MyModal {
    first_input: String,
    second_input: Option<String>,
}
#[poise::command(slash_command)]
pub async fn modal(
    ctx: poise::ApplicationContext<'_, crate::Data, crate::Error>,
) -> Result<(), Error> {
    use poise::Modal as _;

    let data = MyModal::execute(ctx).await?;
    println!("Got data: {:?}", data);

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
