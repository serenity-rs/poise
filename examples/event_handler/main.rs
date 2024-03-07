use std::env::var;
use std::sync::atomic::{AtomicU32, Ordering};

use poise::serenity_prelude as serenity;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    poise_mentions: AtomicU32,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    poise_mentions: AtomicU32::new(0),
                })
            })
        })
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("poise") {
                let old_mentions = data.poise_mentions.fetch_add(1, Ordering::SeqCst);
                new_message
                    .reply(
                        ctx,
                        format!("Poise has been mentioned {} times", old_mentions + 1),
                    )
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
