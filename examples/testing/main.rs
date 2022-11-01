use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
struct Data {}

#[poise::command(slash_command, prefix_command)]
async fn child2(ctx: Context<'_>, _b: bool, _s: String, _i: u32) -> Result<(), Error> {
    ctx.say(ctx.invocation_string()).await?;
    Ok(())
}
#[poise::command(slash_command, prefix_command, subcommands("child2"))]
async fn child1(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
#[poise::command(slash_command, prefix_command, subcommands("child1"))]
async fn parent(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![parent()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").unwrap())
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .user_data_setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id =
                    serenity::GuildId(std::env::var("GUILD_ID").unwrap().parse().unwrap());
                guild_id
                    .set_application_commands(ctx, |b| {
                        *b = poise::builtins::create_application_commands(
                            &framework.options().commands,
                        );
                        b
                    })
                    .await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
