use crate::{Context, Error};

fn autocomplete_name(partial: String) -> impl Iterator<Item = poise::AutocompleteChoice<String>> {
    ["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"]
        .iter()
        .filter(move |name| name.starts_with(&partial))
        .map(|name| poise::AutocompleteChoice {
            name: name.to_string(),
            value: name.to_string(),
        })
}

fn autocomplete_number(_partial: u32) -> impl Iterator<Item = poise::AutocompleteChoice<u32>> {
    // Dummy choices
    [1_u32, 2, 3, 4, 5]
        .iter()
        .map(|&n| poise::AutocompleteChoice {
            name: format!(
                "{} (why do autocomplete choices have a separate label???)",
                n
            ),
            value: n,
        })
}

/// Greet a user. Showcasing autocomplete!
#[poise::command(slash_command)]
pub async fn greet(
    ctx: Context<'_>,
    #[description = "Who to greet"]
    #[autocomplete = "autocomplete_name"]
    name: String,
    #[description = "A number... idk I wanted to test number autocomplete"]
    #[autocomplete = "autocomplete_number"]
    number: Option<u32>,
) -> Result<(), Error> {
    let mut response = format!("Hello {}", name);
    if let Some(number) = number {
        response += &format!("#{}", number);
    }
    response += "!";

    ctx.say(response).await?;
    Ok(())
}
