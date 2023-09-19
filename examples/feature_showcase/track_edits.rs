use crate::{Context, Error};
use poise::serenity_prelude as serenity;

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
                serenity::CreateButton::new("1")
                    .label("button 1")
                    .style(serenity::ButtonStyle::Primary),
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
                serenity::CreateButton::new("2")
                    .label("button 2")
                    .style(serenity::ButtonStyle::Danger),
            ])]),
    )
    .await?;

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
