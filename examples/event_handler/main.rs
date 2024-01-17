use std::env::var;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

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
        .options(poise::FrameworkOptions {
            event_handler: |framework, event| Box::pin(event_handler(framework, event)),
            ..Default::default()
        })
        .build();

    let data = Data {
        poise_mentions: AtomicU32::new(0),
    };

    let client = serenity::ClientBuilder::new(&token, intents)
        .data(Arc::new(data) as _)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    framework: poise::FrameworkContext<'_, Data, Error>,
    event: &serenity::FullEvent,
) -> Result<(), Error> {
    let data = framework.user_data();
    let ctx = framework.serenity_context;

    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.content.to_lowercase().contains("poise") {
                let mentions = data.poise_mentions.load(Ordering::SeqCst) + 1;
                data.poise_mentions.store(mentions, Ordering::SeqCst);
                new_message
                    .reply(ctx, format!("Poise has been mentioned {} times", mentions))
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
