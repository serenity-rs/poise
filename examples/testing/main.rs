mod inherit_checks;
mod misc;

use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                inherit_checks::parent(),
                misc::paginate(),
                misc::div(),
                misc::stringlen(),
                misc::role(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::ArgumentParse { error, .. } => {
                            if let Some(error) = error.downcast_ref::<serenity::RoleParseError>() {
                                println!("Found a RoleParseError: {:?}", error);
                            } else {
                                println!("Not a RoleParseError :(");
                            }
                        }
                        other => poise::builtins::on_error(other).await.unwrap(),
                    }
                })
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").unwrap())
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let guild_id =
                    serenity::GuildId(std::env::var("GUILD_ID").unwrap().parse().unwrap());
                guild_id
                    .set_application_commands(ctx, |b| {
                        *b = poise::builtins::create_application_commands(
                            &framework.options().commands,
                        );
                        b
                    })
                    .await?;
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
