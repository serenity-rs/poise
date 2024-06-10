mod attachment_parameter;
mod autocomplete;
mod bool_parameter;
mod builtins;
mod checks;
mod choice_parameter;
mod code_block_parameter;
mod collector;
mod context_menu;
mod inherit_checks;
mod localization;
mod modal;
mod paginate;
mod panic_handler;
mod parameter_attributes;
mod raw_identifiers;
mod response_with_reply;
mod subcommand_required;
mod subcommands;
mod track_edits;

use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub type Data = ();

#[poise::command(prefix_command, owners_only)]
async fn register_commands(ctx: Context<'_>) -> Result<(), Error> {
    let commands = &ctx.framework().options().commands;
    poise::builtins::register_globally(ctx.http(), commands).await?;

    ctx.say("Successfully registered slash commands!").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let framework_options = poise::FrameworkOptions {
        commands: vec![
            register_commands(),
            attachment_parameter::file_details(),
            attachment_parameter::totalsize(),
            autocomplete::greet(),
            bool_parameter::oracle(),
            #[cfg(feature = "cache")]
            builtins::servers(),
            builtins::help(),
            builtins::pretty_help(),
            checks::shutdown(),
            checks::modonly(),
            checks::delete(),
            checks::ferrisparty(),
            checks::cooldowns(),
            checks::minmax(),
            checks::get_guild_name(),
            checks::only_in_dms(),
            checks::lennyface(),
            checks::permissions_v2(),
            choice_parameter::choice(),
            choice_parameter::inline_choice(),
            choice_parameter::inline_choice_int(),
            code_block_parameter::code(),
            collector::boop(),
            context_menu::user_info(),
            context_menu::echo(),
            inherit_checks::parent_checks(),
            localization::welcome(),
            modal::modal(),
            modal::component_modal(),
            paginate::paginate(),
            panic_handler::div(),
            parameter_attributes::addmultiple(),
            parameter_attributes::voiceinfo(),
            parameter_attributes::say(),
            parameter_attributes::punish(),
            parameter_attributes::stringlen(),
            raw_identifiers::r#move(),
            response_with_reply::reply(),
            subcommands::parent(),
            subcommand_required::parent_subcommand_required(),
            track_edits::test_reuse_response(),
            track_edits::add(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            non_command_message: Some(|_, msg| {
                Box::pin(async move {
                    println!("non command message!: {}", msg.content);
                    Ok(())
                })
            }),
            ..Default::default()
        },
        on_error: |error| {
            Box::pin(async move {
                println!("what the hell");
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
    };

    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(&token, intents)
        .framework(poise::Framework::new(framework_options, true))
        .await;

    client.unwrap().start().await.unwrap()
}
