mod translation;

use poise::serenity_prelude as serenity;
use translation::tr;

pub struct Data {
    translations: translation::Translations,
}

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
    use poise::ChoiceParameter as _;

    ctx.say(format!("<@{}> {}", user.id.0, tr!(ctx, message.name())))
        .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .partial_guild()
        .await
        .ok_or_else(|| tr!(ctx, "not-in-guild-error"))?;

    let response = tr!(ctx, "guild-info",
        name: &*guild.name,
        emojiCount: guild.emojis.len(),
        emojis: guild.emojis.values().map(|e| e.to_string()).collect::<String>(),
        roleCount: guild.roles.len(),
        stickerCount: guild.stickers.len(),
    );

    ctx.say(response).await?;
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

    let mut commands = vec![welcome(), info(), register()];
    let translations = translation::read_ftl().expect("failed to read translation files");
    translation::apply_translations(&translations, &mut commands);

    poise::Framework::builder()
        .token(std::env::var("TOKEN").unwrap())
        .intents(serenity::GatewayIntents::non_privileged())
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .setup(move |_, _, _| Box::pin(async move { Ok(Data { translations }) }))
        .run()
        .await
        .unwrap();
}
