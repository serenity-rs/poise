use poise::Context;
use core::convert::Infallible;

#[poise::command(slash_command)]
pub async fn example<T: Send + Sync>(_ctx: Context<'_, T, Infallible>) -> Result<(), Infallible> {
    Ok(())
}

fn main() {}
