use crate::{Context, Error};
use futures::{Stream, StreamExt};

// Poise supports autocomplete on slash command parameters. You need to provide an autocomplete
// function, which will be called on demand when the user is typing a command.
//
// The first parameter of that function is ApplicationContext or Context, and the second parameter
// is the partial input which the user has typed so far.
//
// As the return value of autocomplete functions, you can return a Stream, an Iterator, or an
// IntoIterator (which includes e.g. Vec<T> and [T; N]).
//
// As the type of the returned items, you can use either the type directly, or wrap it in
// AutocompleteChoice. By wrapping in AutocompleteChoice, you can set a custom name for each choice
// which will be displayed in the Discord UI.

async fn autocomplete_name(_ctx: Context<'_>, partial: String) -> impl Stream<Item = String> {
    futures::stream::iter(&["Amanda", "Bob", "Christian", "Danny", "Ester", "Falk"])
        .filter(move |name| futures::future::ready(name.starts_with(&partial)))
        .map(|name| name.to_string())
}

async fn autocomplete_number(
    _ctx: Context<'_>,
    _partial: u32,
) -> impl Iterator<Item = poise::AutocompleteChoice<u32>> {
    // Dummy choices
    [1_u32, 2, 3, 4, 5]
        .iter()
        .map(|&n| poise::AutocompleteChoice {
            name: format!(
                "{} (why did discord even give autocomplete choices separate labels)",
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
