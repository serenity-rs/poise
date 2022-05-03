use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
struct Data {}

/// Displays your or another user's account creation date
#[poise::command(slash_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or(ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id = serenity::GuildId(todo!("GUILD ID HERE"));
                let commands_builder =
                    poise::builtins::create_application_commands(&framework.options().commands);
                guild_id
                    .set_application_commands(ctx, |b| {
                        *b = commands_builder;
                        b
                    })
                    .await?;

                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
