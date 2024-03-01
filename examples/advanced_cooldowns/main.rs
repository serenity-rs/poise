use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn dynamic_cooldowns(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();

        // You can change the cooldown duration depending on the message author, for example
        let mut cooldown_durations = poise::CooldownConfig::default();
        if ctx.author().id == 472029906943868929 {
            cooldown_durations.user = Some(std::time::Duration::from_secs(10));
        }

        match cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations) {
            Some(remaining) => {
                return Err(format!("Please wait {} seconds", remaining.as_secs()).into())
            }
            None => cooldown_tracker.start_cooldown(ctx.cooldown_context()),
        }
    };

    ctx.say("You successfully invoked the command!").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![dynamic_cooldowns()],
            // This is important! Or else, the command will be marked as invoked before our custom
            // cooldown code has run - even if the command ends up not running!
            manual_cooldowns: true,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let commands = &framework.options().commands;
                poise::builtins::register_globally(&ctx.http, commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::Client::builder(token, serenity::GatewayIntents::non_privileged())
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
