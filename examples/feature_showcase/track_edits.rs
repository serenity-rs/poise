use poise::serenity_prelude as serenity;

use crate::{Context, Error};

#[poise::command(slash_command, prefix_command, reuse_response)]
pub async fn test_reuse_response(ctx: Context<'_>) -> Result<(), Error> {
    let image_url = "https://raw.githubusercontent.com/serenity-rs/serenity/current/logo.png";

    let embed = serenity::CreateEmbed::default()
        .description("embed 1")
        .image(image_url, Some("Serenity logo".into()));

    let buttons =
        [serenity::CreateButton::new("1").label("button 1").style(serenity::ButtonStyle::Primary)];

    let components =
        [serenity::CreateComponent::ActionRow(serenity::CreateActionRow::buttons(&buttons))];
    let reply =
        poise::CreateReply::default().content("message 1").embed(embed).components(&components);

    ctx.send(reply).await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let image_url = "https://raw.githubusercontent.com/serenity-rs/serenity/current/examples/e09_create_message_builder/ferris_eyes.png";
    let embed = serenity::CreateEmbed::default()
        .description("embed 2")
        .image(image_url, Some("Ferris with big eyes".into()));

    let buttons =
        [serenity::CreateButton::new("2").label("button 2").style(serenity::ButtonStyle::Danger)];

    let components =
        [serenity::CreateComponent::ActionRow(serenity::CreateActionRow::buttons(&buttons))];
    let reply =
        poise::CreateReply::default().content("message 2").embed(embed).components(&components);

    ctx.send(reply).await?;
    Ok(())
}

/// Add two numbers
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "First operand"] a: f64,
    #[description = "Second operand"] b: f32,
) -> Result<(), Error> {
    ctx.say(format!("Result: {}", a + b as f64)).await?;

    Ok(())
}
