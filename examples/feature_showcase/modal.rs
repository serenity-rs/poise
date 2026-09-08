use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::small_fixed_array::{FixedArray, FixedString};

#[derive(Debug, poise::Modal)]
#[name = "My Modal"]
struct MyModal {
    #[name = "Text Input"]
    #[description = "This is a text input."]
    #[placeholder = "Your text"]
    text: FixedString<u16>,
    #[name = "Paragraph Input"]
    #[description = "This is a multi-line text input."]
    #[placeholder = "Your long text"]
    #[paragraph]
    paragraph: Option<FixedString<u16>>,
    #[name = "String Select"]
    #[string_select("Option 1", "Option 2", "Option 3")]
    #[string_select_emojis("1️⃣", "2️⃣", "3️⃣")]
    #[string_select_descriptions(
        "This is the first option.",
        "This is the second option.",
        "This is the third option."
    )]
    string_select: FixedArray<String>,
    #[name = "Radio Group"]
    #[radio_group("Option 1", "Option 2")]
    #[radio_group_descriptions("This is the first option.", "This is the second option.")]
    radio_group: Option<FixedString>,
    #[name = "Checkbox"]
    #[checkbox]
    checkbox: bool,
}

/// Tests the Modal trait.
#[poise::command(slash_command)]
pub async fn modal(ctx: poise::ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    use poise::Modal as _;

    let data = MyModal::execute(ctx).await?;
    println!("Got data: {data:?}");

    Ok(())
}

/// Tests the Modal trait with defaults.
#[poise::command(slash_command)]
pub async fn modal_with_defaults(
    ctx: poise::ApplicationContext<'_, Data, Error>,
) -> Result<(), Error> {
    use poise::Modal as _;

    let data = MyModal::execute_with_defaults(
        ctx,
        MyModal {
            text: FixedString::from_static_trunc("My text"),
            paragraph: Some(FixedString::from_static_trunc("My long text")),
            string_select: FixedArray::from_vec_trunc(vec!["Option 1".to_owned()]),
            radio_group: None,
            checkbox: true,
        },
    )
    .await?;
    println!("Got data: {data:?}");

    Ok(())
}

/// Tests the Modal trait with component interactions.
///
/// Should be both prefix and slash to make sure it works without any slash command interaction
/// present.
#[poise::command(prefix_command, slash_command)]
pub async fn component_modal(ctx: crate::Context<'_>) -> Result<(), Error> {
    let buttons = [serenity::CreateButton::new("open_modal")
        .label("Open modal")
        .style(poise::serenity_prelude::ButtonStyle::Success)];

    let components =
        [serenity::CreateComponent::ActionRow(serenity::CreateActionRow::buttons(&buttons))];
    let reply = poise::CreateReply::default()
        .content("Click the button below to open the modal")
        .components(&components);

    ctx.send(reply).await?;

    let serenity_ctx = ctx.serenity_context();
    while let Some(mci) = serenity::ComponentInteractionCollector::new(serenity_ctx)
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == "open_modal")
        .await
    {
        let data =
            poise::execute_modal_on_component_interaction::<MyModal>(serenity_ctx, mci, None, None)
                .await?;

        println!("Got data: {data:?}");
    }
    Ok(())
}
