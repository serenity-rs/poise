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
