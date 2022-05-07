//! Building blocks for common commands like help commands or application command registration
//!
//! This file provides sample commands and utility functions like help menus or error handlers to
//! use as a starting point for the framework.

mod help;
pub use help::*;

use crate::serenity_prelude as serenity;

/// An error handler that prints the error into the console and also into the Discord chat.
/// If the user invoked the command wrong ([`crate::FrameworkError::ArgumentParse`]), the command
/// help is displayed and the user is directed to the help menu.
///
/// Can return an error if sending the Discord error message failed. You can decide for yourself
/// how to handle this, for example:
/// ```rust,no_run
/// # async { let error: poise::FrameworkError<'_, (), &str> = todo!();
/// if let Err(e) = poise::builtins::on_error(error).await {
///     println!("Fatal error while sending error message: {}", e);
/// }
/// # };
/// ```
pub async fn on_error<U, E: std::fmt::Display + std::fmt::Debug>(
    error: crate::FrameworkError<'_, U, E>,
) -> Result<(), serenity::Error> {
    match error {
        crate::FrameworkError::Setup { error } => println!("Error in user data setup: {}", error),
        crate::FrameworkError::Listener { error, event, .. } => println!(
            "User event listener encountered an error on {} event: {}",
            event.name(),
            error
        ),
        crate::FrameworkError::Command { ctx, error } => {
            let error = error.to_string();
            ctx.say(error).await?;
        }
        crate::FrameworkError::ArgumentParse { ctx, input, error } => {
            // If we caught an argument parse error, give a helpful error message with the
            // command explanation if available
            let usage = match ctx.command().multiline_help {
                Some(multiline_help) => multiline_help(),
                None => "Please check the help menu for usage information".into(),
            };
            let response = if let Some(input) = input {
                format!(
                    "**Cannot parse `{}` as argument: {}**\n{}",
                    input, error, usage
                )
            } else {
                format!("**{}**\n{}", error, usage)
            };
            ctx.say(response).await?;
        }
        crate::FrameworkError::CommandStructureMismatch { ctx, description } => {
            println!(
                "Error: failed to deserialize interaction arguments for `/{}`: {}",
                ctx.command.name, description,
            );
        }
        crate::FrameworkError::CommandCheckFailed { ctx, error } => {
            println!(
                "A command check failed in command {} for user {}: {:?}",
                ctx.command().name,
                ctx.author().name,
                error,
            );
        }
        crate::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
        } => {
            let msg = format!(
                "You're too fast. Please wait {} seconds before retrying",
                remaining_cooldown.as_secs()
            );
            ctx.send(|b| b.content(msg).ephemeral(true)).await?;
        }
        crate::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
        } => {
            let msg = format!(
                "Command cannot be executed because the bot is lacking permissions: {}",
                missing_permissions,
            );
            ctx.send(|b| b.content(msg).ephemeral(true)).await?;
        }
        crate::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
        } => {
            let response = if let Some(missing_permissions) = missing_permissions {
                format!(
                    "You're lacking permissions for `{}{}`: {}",
                    ctx.prefix(),
                    ctx.command().name,
                    missing_permissions,
                )
            } else {
                format!(
                    "You may be lacking permissions for `{}{}`. Not executing for safety",
                    ctx.prefix(),
                    ctx.command().name,
                )
            };
            ctx.send(|b| b.content(response).ephemeral(true)).await?;
        }
        crate::FrameworkError::NotAnOwner { ctx } => {
            let response = "Only bot owners can call this command";
            ctx.send(|b| b.content(response).ephemeral(true)).await?;
        }
        crate::FrameworkError::GuildOnly { ctx } => {
            let response = "You cannot run this command in DMs.";
            ctx.send(|b| b.content(response).ephemeral(true)).await?;
        }
        crate::FrameworkError::DmOnly { ctx } => {
            let response = "You cannot run this command outside DMs.";
            ctx.send(|b| b.content(response).ephemeral(true)).await?;
        }
        crate::FrameworkError::NsfwOnly { ctx } => {
            let response = "You cannot run this command outside NSFW channels.";
            ctx.send(|b| b.content(response).ephemeral(true)).await?;
        }
        crate::FrameworkError::DynamicPrefix { error } => {
            println!("Dynamic prefix failed: {}", error);
        }
        crate::FrameworkError::__NonExhaustive => panic!(),
    }

    Ok(())
}

/// An autocomplete function that can be used for the command parameter in your help function.
///
/// See examples/framework_usage for an example
pub async fn autocomplete_command<U, E>(
    ctx: crate::Context<'_, U, E>,
    partial: String,
) -> impl Iterator<Item = String> + '_ {
    ctx.framework()
        .options()
        .commands
        .iter()
        .filter(move |cmd| cmd.name.starts_with(&partial))
        .map(|cmd| cmd.name.to_string())
}

/// Collects all commands into a [`serenity::CreateApplicationCommands`] builder, which can be used
/// to register the commands on Discord
///
/// Also see [`register_application_commands_buttons`] for a ready to use register command
///
/// ```rust,no_run
/// # use poise::serenity_prelude as serenity;
/// # async fn foo<U, E>(ctx: poise::Context<'_, U, E>) -> Result<(), serenity::Error> {
/// let commands = &ctx.framework().options().commands;
/// let create_commands = poise::builtins::create_application_commands(commands);
///
/// serenity::ApplicationCommand::set_global_application_commands(ctx.discord(), |b| {
///     *b = create_commands; // replace the given builder with the one prepared by poise
///     b
/// }).await?;
/// # Ok(()) }
/// ```
pub fn create_application_commands<U, E>(
    commands: &[crate::Command<U, E>],
) -> serenity::CreateApplicationCommands {
    /// We decided to extract context menu commands recursively, despite the subcommand hierarchy
    /// not being preserved. Because it's more confusing to just silently discard context menu
    /// commands if they're not top-level commands.
    /// https://discord.com/channels/381880193251409931/919310428344029265/947970605985189989
    fn recursively_add_context_menu_commands<U, E>(
        builder: &mut serenity::CreateApplicationCommands,
        command: &crate::Command<U, E>,
    ) {
        if let Some(context_menu_command) = command.create_as_context_menu_command() {
            builder.add_application_command(context_menu_command);
        }
        for subcommand in &command.subcommands {
            recursively_add_context_menu_commands(builder, subcommand);
        }
    }

    let mut commands_builder = serenity::CreateApplicationCommands::default();
    for command in commands {
        if let Some(slash_command) = command.create_as_slash_command() {
            commands_builder.add_application_command(slash_command);
        }
        recursively_add_context_menu_commands(&mut commands_builder, command);
    }
    commands_builder
}
/// _Note: you probably want [`register_application_commands_buttons`] instead; it's easier and more
/// powerful_
///
/// Wraps [`create_application_commands`] and adds a bot owner permission check and status messages.
///
/// This function is supposed to be a ready-to-use implementation for a `~register` command of your
/// bot. So if you want, you can copy paste this help message for the command:
///
/// ```text
/// Registers application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
/// ```
pub async fn register_application_commands<U, E>(
    ctx: crate::Context<'_, U, E>,
    global: bool,
) -> Result<(), serenity::Error> {
    let commands = &ctx.framework().options().commands;
    let commands_builder = create_application_commands(commands);

    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    if global {
        ctx.say(format!("Registering {} commands...", commands.len()))
            .await?;
        serenity::ApplicationCommand::set_global_application_commands(ctx.discord(), |b| {
            *b = commands_builder;
            b
        })
        .await?;
    } else {
        let guild_id = match ctx.guild_id() {
            Some(x) => x,
            None => {
                ctx.say("Must be called in guild").await?;
                return Ok(());
            }
        };

        ctx.say(format!("Registering {} commands...", commands.len()))
            .await?;
        guild_id
            .set_application_commands(ctx.discord(), |b| {
                *b = commands_builder;
                b
            })
            .await?;
    }

    ctx.say("Done!").await?;

    Ok(())
}

/// Spawns four buttons to register or delete application commands globally or in the current guild
///
/// Upgraded version of [`register_application_commands`]
pub async fn register_application_commands_buttons<U, E>(
    ctx: crate::Context<'_, U, E>,
) -> Result<(), serenity::Error> {
    let commands = &ctx.framework().options().commands;
    let create_commands = create_application_commands(commands);

    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    let mut msg = ctx
        .send(|m| {
            m.content("Choose what to do with the commands:")
                .components(|c| {
                    c.create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("register.global")
                                .label("Register globally")
                                .style(serenity::ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("unregister.global")
                                .label("Delete globally")
                                .style(serenity::ButtonStyle::Danger)
                        })
                    })
                    .create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("register.guild")
                                .label("Register in guild")
                                .style(serenity::ButtonStyle::Primary)
                        })
                        .create_button(|b| {
                            b.custom_id("unregister.guild")
                                .label("Delete in guild")
                                .style(serenity::ButtonStyle::Danger)
                        })
                    })
                })
        })
        .await?
        .message()
        .await?;

    let interaction = msg
        .await_component_interaction(ctx.discord())
        .collect_limit(1)
        .await;
    msg.edit(ctx.discord(), |b| b.components(|b| b)).await?; // remove buttons after button press
    let pressed_button_id = match &interaction {
        Some(m) => &m.data.custom_id,
        None => {
            ctx.say("You didn't interact in time").await?;
            return Ok(());
        }
    };

    let (register, global) = match &**pressed_button_id {
        "register.global" => (true, true),
        "unregister.global" => (false, true),
        "register.guild" => (true, false),
        "unregister.guild" => (false, false),
        other => {
            log::warn!("unknown register button ID: {:?}", other);
            return Ok(());
        }
    };

    if global {
        if register {
            ctx.say(format!("Registering {} global commands...", commands.len()))
                .await?;
            serenity::ApplicationCommand::set_global_application_commands(ctx.discord(), |b| {
                *b = create_commands;
                b
            })
            .await?;
        } else {
            ctx.say("Unregistering global commands...").await?;
            serenity::ApplicationCommand::set_global_application_commands(ctx.discord(), |b| b)
                .await?;
        }
    } else {
        let guild_id = match ctx.guild_id() {
            Some(x) => x,
            None => {
                ctx.say("Must be called in guild").await?;
                return Ok(());
            }
        };
        if register {
            ctx.say(format!("Registering {} guild commands...", commands.len()))
                .await?;
            guild_id
                .set_application_commands(ctx.discord(), |b| {
                    *b = create_commands;
                    b
                })
                .await?;
        } else {
            ctx.say("Unregistering guild commands...").await?;
            guild_id
                .set_application_commands(ctx.discord(), |b| b)
                .await?;
        }
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
#[cfg(feature = "cache")]
pub async fn servers<U, E>(ctx: crate::Context<'_, U, E>) -> Result<(), serenity::Error> {
    let mut show_private_guilds = false;
    if let crate::Context::Application(_) = ctx {
        if let Ok(app) = ctx.discord().http.get_current_application_info().await {
            if app.owner.id == ctx.author().id {
                show_private_guilds = true;
            }
        }
    }

    /// Stores details of a guild for the purposes of listing it in the bot guild list
    struct Guild {
        /// Name of the guild
        name: String,
        /// Number of members in the guild
        num_members: u64,
        /// Whether the guild is public
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

    ctx.send(|b| b.content(response).ephemeral(show_private_guilds))
        .await?;

    Ok(())
}
