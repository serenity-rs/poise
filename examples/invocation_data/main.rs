//! On request of a user, a system to store data across all phases of a single command invocation
//! was added. See the invocation_data and set_invocation_data methods on Context
//!
//! This module has a test command which stores a dummy payload to check that this string is
//! available in all phases of command execution

type Error = &'static str;
type Context<'a> = poise::Context<'a, (), Error>;

async fn my_check(ctx: Context<'_>) -> Result<bool, Error> {
    println!(
        "In command specific check: {:?}",
        ctx.invocation_data::<&str>().await.as_deref()
    );

    Ok(true)
}

async fn my_autocomplete(ctx: Context<'_>, _: u32) -> impl Iterator<Item = u32> {
    println!(
        "In autocomplete: {:?}",
        ctx.invocation_data::<&str>().await.as_deref()
    );

    std::iter::empty()
}

/// Test command to ensure that invocation_data works
#[poise::command(prefix_command, slash_command, check = "my_check")]
pub async fn invocation_data_test(
    ctx: Context<'_>,
    #[description = "Whether this command should succeed (yes if 0, no otherwise)"]
    #[autocomplete = "my_autocomplete"]
    should_succeed: u32,
) -> Result<(), Error> {
    println!(
        "In command: {:?}",
        ctx.invocation_data::<&str>().await.as_deref()
    );

    if should_succeed > 0 {
        Ok(())
    } else {
        Err("")
    }
}

#[tokio::main]
async fn main() {
    poise::Framework::build()
        .token(std::env::var("TOKEN").unwrap())
        .user_data_setup(move |ctx, _, framework| {
            Box::pin(async move {
                poise::serenity_prelude::GuildId(703332075914264606)
                    .set_application_commands(ctx, |b| {
                        *b = poise::samples::create_application_commands(
                            &framework.options().commands,
                        );
                        b
                    })
                    .await
                    .unwrap();
                Ok(())
            })
        })
        .options(poise::FrameworkOptions {
            pre_command: |ctx| {
                Box::pin(async move {
                    println!(
                        "In pre_command: {:?}",
                        ctx.invocation_data::<&str>().await.as_deref()
                    );
                })
            },
            command_check: Some(|ctx| {
                Box::pin(async move {
                    // Global command check is the first callback that's invoked, so let's set the
                    // data here
                    println!("Writing invocation data!");
                    ctx.set_invocation_data("hello").await;

                    println!(
                        "In global check: {:?}",
                        ctx.invocation_data::<&str>().await.as_deref()
                    );

                    Ok(true)
                })
            }),
            post_command: |ctx| {
                Box::pin(async move {
                    println!(
                        "In post_command: {:?}",
                        ctx.invocation_data::<&str>().await.as_deref()
                    );
                })
            },
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        poise::FrameworkError::Command { ctx, .. } => {
                            println!(
                                "In on_error: {:?}",
                                ctx.invocation_data::<&str>().await.as_deref()
                            );
                        }
                        err => poise::samples::on_error(err).await.unwrap(),
                    }
                })
            },

            commands: vec![invocation_data_test()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
        .await
        .unwrap();
}
