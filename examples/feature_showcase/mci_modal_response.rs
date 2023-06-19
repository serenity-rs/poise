use crate::{Data, Error};

#[derive(Debug, poise::Modal)]
#[allow(dead_code)] // fields only used for Debug print
struct MyModal {
    first_input: String,
    second_input: Option<String>,
}
#[poise::command(slash_command)]
pub async fn mci_modal_response(
    ctx: poise::ApplicationContext<'_, Data, Error>,
) -> Result<(), Error> {
    use poise::Modal as _;

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
        let data = MyModal::execute_component_interaction(ctx, mci).await?;
        println!("Got data: {:?}", data);
    }
    Ok(())
}
