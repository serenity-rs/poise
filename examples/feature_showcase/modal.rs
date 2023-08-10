use crate::{Data, Error};

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
    ctx.send(|m| {
        m.content("Click the button below to open the modal")
            .components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| {
                        b.custom_id("open_modal")
                            .label("Open modal")
                            .style(poise::serenity_prelude::ButtonStyle::Success)
                    })
                })
            })
    })
    .await?;

    while let Some(mci) =
        poise::serenity_prelude::CollectComponentInteraction::new(ctx.serenity_context())
            .timeout(std::time::Duration::from_secs(120))
            .filter(move |mci| mci.data.custom_id == "open_modal")
            .await
    {
        let data =
            poise::execute_modal_on_component_interaction::<MyModal>(ctx, mci, None, None).await?;
        println!("Got data: {:?}", data);
    }
    Ok(())
}
