//! Building blocks for common commands like help commands or application command registration
//!
//! This file provides sample commands and utility functions like help menus or error handlers to
//! use as a starting point for the framework.

mod help;
pub use help::*;

use crate::serenity_prelude as serenity;
type BoxErrorSendSync = Box<dyn std::error::Error + Send + Sync>;

/// An error handler that prints the error into the console and also into the Discord chat.
/// If the user invoked the command wrong
/// (i.e. an [`crate::ArgumentParseError`]), the command help is displayed and the user is directed
/// to the help menu.
pub async fn on_error<D>(e: BoxErrorSendSync, ctx: crate::ErrorContext<'_, D, BoxErrorSendSync>) {
    println!("Encountered an error: {:?}", e);
    match ctx {
        crate::ErrorContext::Command(ctx) => {
            let user_error_msg = if let Some(crate::ArgumentParseError(e)) = e.downcast_ref() {
                // If we caught an argument parse error, give a helpful error message with the
                // command explanation if available

                let mut usage = "Please check the help menu for usage information".into();
                if let Some(multiline_help) = &ctx.command().id().multiline_help {
                    usage = multiline_help();
                }
                format!("**{}**\n{}", e, usage)
            } else {
                e.to_string()
            };
            if let Err(e) = ctx.ctx().say(user_error_msg).await {
                println!("Error while user command error: {}", e);
            }
        }
        crate::ErrorContext::Listener(event) => {
            println!("Error in listener while processing {:?}: {}", event, e)
        }
        crate::ErrorContext::Autocomplete(err_ctx) => {
            let ctx = err_ctx.ctx;
            println!(
                "Error in autocomplete callback for command {:?}: {}",
                ctx.command.slash_or_context_menu_name(),
                e
            )
        }
        crate::ErrorContext::Setup => println!("Setup failed: {}", e),
    }
}

/// An autocomplete function that can be used for the command parameter in your help function.
///
/// See examples/framework_usage for an example
pub async fn autocomplete_command<U, E>(
    ctx: crate::Context<'_, U, E>,
    partial: String,
) -> impl Iterator<Item = String> + '_ {
    // We only consider prefix commands here because, bad as it is, that's what other builtins
    // do to. For example the help command only shows commands that have a prefix version.
    // Once a better command structure design is adopted, this issue should be solved
    ctx.framework()
        .options()
        .prefix_options
        .commands
        .iter()
        .filter(move |cmd| cmd.command.name.starts_with(&partial))
        .map(|cmd| cmd.command.name.to_string())
}

/// Generic function to register application commands, either globally or in a guild. Only bot
/// owners can register globally, only guild owners can register in guild.
///
/// If you want, you can copy paste this help message:
///
/// ```ignore
/// Register application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
/// ```
pub async fn register_application_commands<U, E>(
    ctx: crate::Context<'_, U, E>,
    global: bool,
) -> Result<(), serenity::Error> {
    let mut commands_builder = serenity::CreateApplicationCommands::default();
    let commands = &ctx.framework().options().application_options.commands;
    for cmd in commands {
        commands_builder.create_application_command(|f| cmd.create(f));
    }
    let commands_builder = serenity::json::Value::Array(commands_builder.0);

    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if global {
        if !is_bot_owner {
            ctx.say("Can only be used by bot owner").await?;
            return Ok(());
        }

        ctx.say(format!("Registering {} commands...", commands.len()))
            .await?;
        ctx.discord()
            .http
            .create_global_application_commands(&commands_builder)
            .await?;
    } else {
        let guild = match ctx.guild() {
            Some(x) => x,
            None => {
                ctx.say("Must be called in guild").await?;
                return Ok(());
            }
        };
        let is_guild_owner = ctx.author().id == guild.owner_id;

        if !is_guild_owner && !is_bot_owner {
            ctx.say("Can only be used by server owner").await?;
            return Ok(());
        }

        ctx.say(format!("Registering {} commands...", commands.len()))
            .await?;
        ctx.discord()
            .http
            .create_guild_application_commands(guild.id.0, &commands_builder)
            .await?;
    }

    ctx.say("Done!").await?;

    Ok(())
}

/// Lists servers of which the bot is a member of, including their member counts, sorted
/// descendingly by member count.
///
/// Non-[public](https://support.discord.com/hc/en-us/articles/360030843331-Enabling-Server-Discovery)
/// guilds are hidden to preserve privacy. When the command is invoked by the bot
/// owner as an application command, the response will be made ephemeral and private guilds are
/// unhidden.
///
/// Example:
/// > I am currently in three servers!
/// > - **A public server** (7123 members)
/// > - [3 private servers with 456 members total]
pub async fn servers<U, E>(ctx: crate::Context<'_, U, E>) -> Result<(), serenity::Error> {
    let mut show_private_guilds = false;
    if let crate::Context::Application(_) = ctx {
        if let Ok(app) = ctx.discord().http.get_current_application_info().await {
            if app.owner.id == ctx.author().id {
                show_private_guilds = true;
            }
        }
    }

    struct Guild {
        name: String,
        num_members: u64,
        is_public: bool,
    }

    let guild_ids = ctx.discord().cache.guilds();
    let mut guilds = guild_ids
        .into_iter()
        .filter_map(|guild_id| {
            ctx.discord().cache.guild_field(guild_id, |guild| Guild {
                name: guild.name.clone(),
                num_members: guild.member_count,
                is_public: guild.features.iter().any(|x| x == "DISCOVERABLE"),
            })
        })
        .collect::<Vec<_>>();
    guilds.sort_by_key(|guild| u64::MAX - guild.num_members); // descending sort

    let mut num_private_guilds = 0;
    let mut num_private_guild_members = 0;
    let mut response = format!("I am currently in {} servers!\n", guilds.len());
    for guild in guilds {
        if guild.is_public || show_private_guilds {
            response += &format!("- **{}** ({} members)\n", guild.name, guild.num_members);
        } else {
            num_private_guilds += 1;
            num_private_guild_members += guild.num_members;
        }
    }
    if num_private_guilds > 0 {
        response += &format!(
            "- [{} private servers with {} members total]\n",
            num_private_guilds, num_private_guild_members
        );
    }

    if show_private_guilds {
        response += "\n_Showing private guilds because you are the bot owner_\n";
    }

    ctx.send(|f| f.content(response).ephemeral(show_private_guilds))
        .await?;

    Ok(())
}
