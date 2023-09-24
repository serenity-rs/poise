//! If you need it, poise-annotated command functions can also be generic over the user data type
//! or error type
//!
//! The original use case for this feature was to have the same command in two different bots

#[poise::command(slash_command)]
pub async fn example<U: Sync, E>(ctx: poise::Context<'_, U, E>) -> Result<(), E> {
    ctx.say(format!(
        "My user data type is {} and the error type is {}",
        std::any::type_name::<U>(),
        std::any::type_name::<E>()
    ))
    .await
    .unwrap();

    Ok(())
}

#[tokio::main]
async fn main() {
    let _example1 = example::<(), ()>();
    let _example2 = example::<String, Box<dyn std::error::Error>>();
}
