use std::fmt::Write as _;

use poise::serenity_prelude as serenity;

use crate::{Context, Error};

// Poise supports autocomplete on slash command parameters. You need to provide an autocomplete
// function, which will be called on demand when the user is typing a command.
//
// The first parameter of that function is ApplicationContext or Context, and the second parameter
// is a &str of the partial input which the user has typed so far.
//
// As the return value of autocomplete functions, you must return `serenity::CreateAutocompleteResponse`.

async fn autocomplete_name<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> serenity::CreateAutocompleteResponse {
    let choices = ["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"]
        .into_iter()
        .filter(move |name| name.starts_with(partial))
        .map(serenity::AutocompleteChoice::from)
        .collect();

    serenity::CreateAutocompleteResponse::new().set_choices(choices)
}

async fn autocomplete_number(
    _ctx: Context<'_>,
    _partial: &str,
) -> serenity::CreateAutocompleteResponse {
    // Dummy choices
    let choices = [1_u32, 2, 3, 4, 5].iter().map(|&n| {
        serenity::AutocompleteChoice::new(
            format!("{n} (why did discord even give autocomplete choices separate labels)"),
            n,
        )
    });

    serenity::CreateAutocompleteResponse::new().set_choices(choices.collect())
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
        let _ = write!(response, "#{}", number);
    }
    response += "!";

    ctx.say(response).await?;
    Ok(())
}
