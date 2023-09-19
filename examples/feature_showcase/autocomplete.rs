use crate::{Context, Error};
use futures::{Stream, StreamExt};
use std::fmt::Write as _;

// Poise supports autocomplete on slash command parameters. You need to provide an autocomplete
// function, which will be called on demand when the user is typing a command.
//
// The first parameter of that function is ApplicationContext or Context, and the second parameter
// is a &str of the partial input which the user has typed so far.
//
// As the return value of autocomplete functions, you can return a Stream, an Iterator, or an
// IntoIterator like Vec<T> and [T; N].
//
// The returned collection type must be a &str/String (or number, if you're implementing
// autocomplete on a number type). Wrap the type in poise::AutocompleteChoice to set a custom label
// for each choice which will be displayed in the Discord UI.
//
// Example function return types (assuming non-number parameter -> autocomplete choices are string):
// - `-> impl Stream<String>`
// - `-> Vec<String>`
// - `-> impl Iterator<String>`
// - `-> impl Iterator<&str>`
// - `-> impl Iterator<poise::AutocompleteChoice<&str>>`

async fn autocomplete_name<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(&["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}

async fn autocomplete_number(
    _ctx: Context<'_>,
    _partial: &str,
) -> impl Iterator<Item = poise::AutocompleteChoice<u32>> {
    // Dummy choices
    [1_u32, 2, 3, 4, 5].iter().map(|&n| {
        poise::AutocompleteChoice::new_with_value(
            format!(
                "{} (why did discord even give autocomplete choices separate labels)",
                n
            ),
            n,
        )
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
        let _ = write!(response, "#{}", number);
    }
    response += "!";

    ctx.say(response).await?;
    Ok(())
}
