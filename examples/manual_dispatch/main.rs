//! poise::Framework handles client creation and event handling for you. Alternatively, you can
//! do that yourself and merely forward the events you receive to poise. This example shows how.
//!
//! Note: this example configures no designated prefix. Mention the bot as a prefix instead. For
//! that to work, please adjust the bot ID below to your bot, for the mention parsing to work.

use poise::serenity_prelude as serenity;

type Error = serenity::Error;

#[poise::command(prefix_command)]
async fn ping(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

struct Handler {
    options: poise::FrameworkOptions<(), Error>,
    shard_manager: std::sync::Mutex<Option<std::sync::Arc<serenity::ShardManager>>>,
}
#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn message(&self, serenity_context: serenity::Context, new_message: serenity::Message) {
        // FrameworkContext contains all data that poise::Framework usually manages
        let shard_manager = (*self.shard_manager.lock().unwrap()).clone().unwrap();
        let framework_data = poise::FrameworkContext {
            serenity_context: &serenity_context,
            options: &self.options,
            shard_manager: &shard_manager,
        };

        let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
        let trigger = poise::MessageDispatchTrigger::MessageCreate;
        let mut parent_commands = Vec::new();

        let res = poise::dispatch_message(
            framework_data,
            &new_message,
            trigger,
            &invocation_data,
            &mut parent_commands,
        );

        if let Err(err) = res.await {
            err.handle(&self.options).await;
        }
    }

    // For slash commands or edit tracking to work, forward interaction_create and message_update
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();
    let mut handler = Handler {
        options: poise::FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        },
        shard_manager: std::sync::Mutex::new(None),
    };
    poise::set_qualified_names(&mut handler.options.commands); // some setup

    let handler = std::sync::Arc::new(handler);
    let mut client = serenity::Client::builder(&token, intents)
        .event_handler::<Handler>(handler.clone())
        .await?;

    *handler.shard_manager.lock().unwrap() = Some(client.shard_manager.clone());
    client.start().await?;

    Ok(())
}
