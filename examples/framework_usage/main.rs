mod commands;
mod context_menu;

use poise::serenity_prelude as serenity;
use std::{collections::HashMap, env::var, sync::Mutex, time::Duration};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type PrefixContext<'a> = poise::PrefixContext<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
    owner_id: serenity::UserId,
}

/// Show this help menu
#[poise::command(track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    poise::defaults::help(
        ctx,
        command.as_deref(),
        "This is an example bot made to showcase features of my custom Discord bot framework",
        poise::defaults::HelpResponseMode::Ephemeral,
    )
    .await?;
    Ok(())
}

async fn is_owner(ctx: crate::PrefixContext<'_>) -> Result<bool, Error> {
    Ok(ctx.msg.author.id == ctx.data.owner_id)
}

/// Register slash commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(check = "is_owner", hide_in_help)]
async fn register(ctx: PrefixContext<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::defaults::register_slash_commands(ctx, global).await?;

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
async fn main() -> Result<(), Error> {
    let mut options = poise::FrameworkOptions {
        prefix_options: poise::PrefixFrameworkOptions {
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },
        on_error: |error, ctx| Box::pin(on_error(error, ctx)),
        ..Default::default()
    };

    options.command(help(), |f| f);
    options.command(register(), |f| f);
    options.command(commands::vote(), |f| f);
    options.command(commands::getvotes(), |f| f);
    options.command(commands::add(), |f| f);
    options.command(commands::choice(), |f| f);
    options.command(commands::boop(), |f| f);
    options.command(context_menu::user_info(), |f| f);

    let framework = poise::Framework::new(
        "~".to_owned(), // prefix
        serenity::ApplicationId(var("APPLICATION_ID")?.parse()?),
        move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                    owner_id: serenity::UserId(var("OWNER_ID")?.parse()?),
                })
            })
        },
        options,
    );
    framework
        .start(serenity::ClientBuilder::new(&var("TOKEN")?))
        .await?;

    Ok(())
}
