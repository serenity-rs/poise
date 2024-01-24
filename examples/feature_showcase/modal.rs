use crate::{Data, Error};
use poise::serenity_prelude as serenity;

#[derive(Debug, poise::Modal)]
#[allow(dead_code)] // fields only used for Debug print
struct MyModal {
    first_input: String,
    second_input: Option<String>,
}
#[poise::command(slash_command)]
pub async fn modal(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    use poise::Modal as _;

    let data = MyModal::execute(ctx).await?;
    println!("Got data: {:?}", data);

    Ok(())
}

/// Tests the Modal trait with component interactions.
///
/// Should be both prefix and slash to make sure it works without any slash command interaction
/// present.
#[poise::command(prefix_command, slash_command)]
pub async fn component_modal(ctx: crate::Context<'_>) -> Result<(), Error> {
    let reply = {
        let components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("open_modal")
                .label("Open modal")
                .style(poise::serenity_prelude::ButtonStyle::Success),
        ])];

        poise::CreateReply::default()
            .content("Click the button below to open the modal")
            .components(components)
    };

    ctx.send(reply).await?;

    let serenity_ctx = ctx.serenity_context();
    let shard_messenger = &serenity_ctx.shard;
    while let Some(mci) = serenity::ComponentInteractionCollector::new(shard_messenger.clone())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == "open_modal")
        .await
    {
        let data =
            poise::execute_modal_on_component_interaction::<MyModal>(serenity_ctx, mci, None, None)
                .await?;

        println!("Got data: {:?}", data);
    }
    Ok(())
}
