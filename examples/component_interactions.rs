use std::{env, time::Duration};
use uuid::Uuid;

use poise::{EditTracker, ErrorContext, FrameworkOptions, PrefixFrameworkOptions};

use serenity::collector::component_interaction_collector::CollectComponentInteraction;
use serenity::model::prelude::{
    message_component::ButtonStyle, ApplicationId, InteractionResponseType, UserId,
};

type Error = Box<dyn std::error::Error + Send + Sync>;

type Context<'a> = poise::Context<'a, Data, Error>;
type PrefixContext<'a> = poise::PrefixContext<'a, Data, Error>;

struct Data {
    //votes: Mutex<HashMap<String, u32>>,
    owner_id: UserId,
}

/// Pong!
#[poise::command(slash_command, track_edits)]
async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error> {
    poise::say_reply(ctx, "Pong!").await?;

    Ok(())
}

/// Boop the bot!
#[poise::command(slash_command, track_edits)]
async fn boop(ctx: Context<'_>) -> Result<(), Error> {
    let mut boop_count = 1;
    let uuid_boop = Uuid::new_v4().to_string();

    poise::send_reply(ctx, |m| {
        m.content("I want some boops!".to_string());
        m.components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.style(ButtonStyle::Primary);
                    b.label("Boop me!");
                    b.custom_id(&uuid_boop);
                    b
                })
            })
        });
        m
    })
    .await?;

    loop {
        let mov_uuid_boop = uuid_boop.clone();
        let mci = CollectComponentInteraction::new(ctx.discord())
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .timeout(Duration::from_secs(120))
            .filter(move |mci| mci.data.custom_id == mov_uuid_boop)
            .await;

        if let Some(mci) = mci {
            let mut msg = mci.message.clone().regular().unwrap();
            msg.edit(ctx.discord(), |m| {
                m.content(format!("Boop count: {}", boop_count))
            })
            .await?;

            boop_count += 1;

            mci.create_interaction_response(ctx.discord(), |ir| {
                ir.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await?;
        } else {
            println!("Collector returned None, returning.");
            break;
        }
    }

    Ok(())
}

/// Register slash commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
#[poise::command(check = "is_owner", hide_in_help)]
async fn register(ctx: PrefixContext<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::defaults::register_slash_commands(ctx, global).await?;

    Ok(())
}

async fn is_owner(ctx: PrefixContext<'_>) -> Result<bool, Error> {
    Ok(ctx.msg.author.id == ctx.data.owner_id)
}

async fn on_error(error: Error, ctx: ErrorContext<'_, Data, Error>) {
    match ctx {
        ErrorContext::Setup => panic!("Failed to start bot: {:?}", error),
        ErrorContext::Command(ctx) => {
            println!("Error in command `{}`: {:?}", ctx.command().name(), error)
        }
        _ => println!("Other error: {:?}", error),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut options = FrameworkOptions {
        prefix_options: PrefixFrameworkOptions {
            edit_tracker: Some(EditTracker::for_timespan(Duration::from_secs(500))),
            ..Default::default()
        },
        on_error: |error, ctx| Box::pin(on_error(error, ctx)),
        ..Default::default()
    };

    options.command(ping(), |f| f);
    options.command(boop(), |f| f);
    options.command(register(), |f| f);

    let framework = poise::Framework::new(
        ",".to_owned(), // prefix
        ApplicationId(881987798398283806),
        move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    //votes: Mutex::new(HashMap::new()),
                    owner_id: UserId(182891574139682816),
                })
            })
        },
        options,
    );

    framework
        .start(serenity::client::ClientBuilder::new(&env::var(
            "DISCORD_TOKEN",
        )?))
        .await?;

    Ok(())
}
