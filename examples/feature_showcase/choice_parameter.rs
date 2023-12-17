use crate::{Context, Error};

#[derive(Debug, poise::ChoiceParameter)]
pub enum MyStringChoice {
    #[name = "The first choice"]
    A,
    #[name = "The second choice"]
    #[name = "A single choice can have multiple names"]
    B,
    // If no name is given, the variant name is used
    C,
}

/// Dummy command to test slash command choice parameters
#[poise::command(prefix_command, slash_command)]
pub async fn choice(
    ctx: Context<'_>,
    #[description = "The choice you want to choose"] choice: MyStringChoice,
) -> Result<(), Error> {
    ctx.say(format!("You entered {:?}", choice)).await?;
    Ok(())
}

// For simple choices, you can also declare the options inline
//
// Features: supports duplicate options and theoretically any type that implements Display
//
// Limitations: due to macro limitations (partially self-imposed, partially external), poise
// currently does not support Options parameters, and only supports parameter types that can be
// constructed from a literal (https://doc.rust-lang.org/reference/expressions/literal-expr.html).

#[poise::command(slash_command)]
pub async fn inline_choice(
    ctx: Context<'_>,
    #[description = "Which continent are you from"]
    #[choices("Europe", "Asia", "Africa", "America", "Australia", "Antarctica")]
    continent: &'static str,
) -> Result<(), Error> {
    ctx.say(format!("{} is a great continent!", continent))
        .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn inline_choice_int(
    ctx: Context<'_>,
    #[description = "Choose a number"]
    #[choices(1, 2, 3, 4, 5, 4, 3, 2, 1)]
    number: u32,
) -> Result<(), Error> {
    ctx.say(format!("You chose {}... for better or for worse", number))
        .await?;
    Ok(())
}
