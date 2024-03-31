use poise::builtins::get_replied_user;

use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's avatar.
#[poise::command(slash_command, prefix_command)]
async fn avatar(
    ctx: Context<'_>,
    #[description = "Selected user (you can also use message replies)"] user: Option<
        serenity::User,
    >,
) -> Result<(), Error> {
    let replied_user = user.or(get_replied_user(ctx));
    let u = replied_user.as_ref().unwrap_or(ctx.author());

    let response = u.face().replace(".webp", ".png");
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![avatar()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
