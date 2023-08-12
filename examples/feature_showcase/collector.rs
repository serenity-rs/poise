use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Boop the bot!
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn boop(ctx: Context<'_>) -> Result<(), Error> {
    let uuid_boop = ctx.id();

    ctx.send(
        poise::CreateReply::default()
            .content("I want some boops!")
            .components(vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(uuid_boop.to_string())
                    .label("Boop me!")
                    .style(serenity::ButtonStyle::Primary),
            ])]),
    )
    .await?;

    let mut boop_count = 0;
    while let Some(mci) = serenity::ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == uuid_boop.to_string())
        .await
    {
        boop_count += 1;

        let mut msg = mci.message.clone();

        msg.edit(
            ctx,
            serenity::EditMessage::default().content(format!("Boop count: {}", boop_count)),
        )
        .await?;

        mci.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge)
            .await?;
    }

    Ok(())
}
