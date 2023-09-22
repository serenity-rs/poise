use poise::Context;
use core::convert::Infallible;

#[poise::command(slash_command)]
pub async fn example(_ctx: Context<'_, (), Infallible>) -> Result<(), Infallible> {
    Ok(())
}

fn main() {}
