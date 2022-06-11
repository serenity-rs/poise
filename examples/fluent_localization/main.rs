mod translation;

use poise::serenity_prelude as serenity;
use translation::get;

pub struct Data {
    translations: std::collections::HashMap<String, FluentBundle>,
}

type FluentBundle = fluent::bundle::FluentBundle<
    fluent::FluentResource,
    intl_memoizer::concurrent::IntlLangMemoizer,
>;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(poise::ChoiceParameter)]
pub enum WelcomeChoice {
    Ask,
    GoodPerson,
    Controller,
    Coffee,
}

#[poise::command(slash_command)]
pub async fn welcome(
    ctx: Context<'_>,
    user: serenity::User,
    message: WelcomeChoice,
) -> Result<(), Error> {
    ctx.say(format!("<@{}> {}", user.id.0, get(ctx, message.name())))
        .await?;
    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut commands = vec![welcome(), register()];
    let translations = translation::read_ftl().expect("failed to read translation files");
    translation::apply_translations(&translations, &mut commands);

    poise::Framework::build()
        .token(std::env::var("TOKEN").unwrap())
        .intents(serenity::GatewayIntents::non_privileged())
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .user_data_setup(move |_, _, _| Box::pin(async move { Ok(Data { translations }) }))
        .run()
        .await
        .unwrap();
}
