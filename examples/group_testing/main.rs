use poise::{serenity_prelude as serenity, Command, CommandGroup};
use std::{env::var, sync::Arc, time::Duration, vec};
// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {}

/// A Group with one command
struct TestOneCommand {}

#[poise::group(category = "One")]
impl TestOneCommand {
    /// Say hello
    #[poise::command(slash_command, prefix_command, rename = "hello")]
    async fn say_hello(ctx: Context<'_>) -> Result<(), Error> {
        let name = ctx.author();
        ctx.say(format!("Hello, {}", name)).await?;
        Ok(())
    }
}

/// A Group with multiple commands
struct TestMultipleCommands {}

#[poise::group(category = "Multiple")]
impl TestMultipleCommands {
    /// Add one to a number
    #[poise::command(slash_command, prefix_command, rename = "plus")]
    async fn add_one(ctx: Context<'_>, number: u32) -> Result<(), Error> {
        let add_result = number.wrapping_add(1);
        ctx.say(format!("{number} + 1 = {add_result}")).await?;
        Ok(())
    }

    /// Take one from a number
    #[poise::command(slash_command, prefix_command, rename = "minus")]
    async fn subtract_one(ctx: Context<'_>, number: u32) -> Result<(), Error> {
        let add_result = number.wrapping_sub(1);
        ctx.say(format!("{number} - 1 = {add_result}")).await?;
        Ok(())
    }
}

// Handlers
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            eprintln!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                eprintln!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    // println!("{:#?}", Test::commands());
    let commands: Vec<Command<Data, Error>> = TestOneCommand::commands()
        .into_iter()
        .chain(TestMultipleCommands::commands().into_iter())
        .collect();

    let options = poise::FrameworkOptions {
        commands,
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("--".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .options(options)
        .build();
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}
