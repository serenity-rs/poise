mod autocomplete;
mod commands;
mod context_menu;

use std::{collections::HashMap, env::var, sync::Mutex, time::Duration};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        "This is an example bot made to showcase features of my custom Discord bot framework",
        poise::builtins::HelpResponseMode::Ephemeral,
    )
    .await?;
    Ok(())
}

/// Register application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await?;

    Ok(())
}

async fn on_error(error: Error, ctx: poise::ErrorContext<'_, Data, Error>) {
    match ctx {
        poise::ErrorContext::Setup => panic!("Failed to start bot: {:?}", error),
        poise::ErrorContext::Command(ctx) => {
            println!("Error in command `{}`: {:?}", ctx.command().name(), error)
        }
        _ => println!("Other error: {:?}", error),
    }
}

#[tokio::main]
async fn main() {
    let options = poise::FrameworkOptions {
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot"),
                poise::Prefix::Literal("hey bot,"),
            ],
            ..Default::default()
        },
        on_error: |error, ctx| Box::pin(on_error(error, ctx)),
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().unwrap().name());
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().unwrap().name());
            })
        },
        ..Default::default()
    };

    poise::Framework::build()
        .token(var("TOKEN").expect("Missing `TOKEN` env var, see README for more information."))
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .command(help(), |f| f)
        .command(register(), |f| f)
        .command(commands::vote(), |f| f)
        .command(commands::getvotes(), |f| f)
        .command(commands::add(), |f| f)
        .command(commands::choice(), |f| f)
        .command(commands::boop(), |f| f)
        .command(context_menu::user_info(), |f| f)
        .command(context_menu::echo(), |f| f)
        .command(autocomplete::greet(), |f| f)
        .run()
        .await
        .unwrap();
}
