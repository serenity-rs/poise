//! Building blocks for common commands like help commands or application command registration
//!
//! This file provides sample commands and utility functions like help menus or error handlers to
//! use as a starting point for the framework.

mod help;
pub use help::*;

mod register;
pub use register::*;

use crate::serenity_prelude as serenity;

/// Utility function to avoid verbose
/// `ctx.send(crate::CreateReply::default().content(...).ephemeral(...))`
async fn say_ephemeral<U, E>(
    ctx: crate::Context<'_, U, E>,
    msg: &str,
    ephemeral: bool,
) -> Result<(), serenity::Error> {
    ctx.send(
        crate::CreateReply::default()
            .content(msg)
            .ephemeral(ephemeral),
    )
    .await?;
    Ok(())
}

/// An error handler that logs errors either via the [`log`] crate or via a Discord message. Set
/// up a logger (e.g. `env_logger::init()`) to see the logged errors from this method.
///
/// If the user invoked the command wrong ([`crate::FrameworkError::ArgumentParse`]), the command
/// help is displayed and the user is directed to the help menu.
///
/// Can return an error if sending the Discord error message failed. You can decide for yourself
/// how to handle this, for example:
/// ```rust,no_run
/// # async { let error: poise::FrameworkError<'_, (), &str> = todo!();
/// if let Err(e) = poise::builtins::on_error(error).await {
///     log::error!("Fatal error while sending error message: {}", e);
/// }
/// # };
/// ```
pub async fn on_error<U, E: std::fmt::Display + std::fmt::Debug>(
    error: crate::FrameworkError<'_, U, E>,
) -> Result<(), serenity::Error> {
    match error {
        crate::FrameworkError::Setup { error, .. } => {
            log::error!("Error in user data setup: {}", error);
        }
        crate::FrameworkError::Listener { error, event, .. } => log::error!(
            "User event listener encountered an error on {} event: {}",
            event.snake_case_name(),
            error
        ),
        crate::FrameworkError::Command { ctx, error } => {
            let error = error.to_string();
            ctx.say(error).await?;
        }
        crate::FrameworkError::ArgumentParse { ctx, input, error } => {
            // If we caught an argument parse error, give a helpful error message with the
            // command explanation if available
            let usage = match ctx.command().help_text {
                Some(help_text) => help_text(),
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
            log::error!(
                "Error: failed to deserialize interaction arguments for `/{}`: {}",
                ctx.command.name,
                description,
            );
        }
        crate::FrameworkError::CommandCheckFailed { ctx, error } => {
            log::error!(
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
            say_ephemeral(ctx, &msg, true).await?;
        }
        crate::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
        } => {
            let msg = format!(
                "Command cannot be executed because the bot is lacking permissions: {}",
                missing_permissions,
            );
            say_ephemeral(ctx, &msg, true).await?;
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
            say_ephemeral(ctx, &response, true).await?;
        }
        crate::FrameworkError::NotAnOwner { ctx } => {
            let response = "Only bot owners can call this command";
            say_ephemeral(ctx, response, true).await?;
        }
        crate::FrameworkError::GuildOnly { ctx } => {
            let response = "You cannot run this command in DMs.";
            say_ephemeral(ctx, response, true).await?;
        }
        crate::FrameworkError::DmOnly { ctx } => {
            let response = "You cannot run this command outside DMs.";
            say_ephemeral(ctx, response, true).await?;
        }
        crate::FrameworkError::NsfwOnly { ctx } => {
            let response = "You cannot run this command outside NSFW channels.";
            say_ephemeral(ctx, response, true).await?;
        }
        crate::FrameworkError::DynamicPrefix { error, msg, .. } => {
            log::error!(
                "Dynamic prefix failed for message {:?}: {}",
                msg.content,
                error
            );
        }
        crate::FrameworkError::UnknownCommand {
            msg_content,
            prefix,
            ..
        } => {
            log::warn!(
                "Recognized prefix `{}`, but didn't recognize command name in `{}`",
                prefix,
                msg_content,
            );
        }
        crate::FrameworkError::UnknownInteraction { interaction, .. } => {
            log::warn!(
                "received unknown interaction \"{}\"",
                interaction.data().name
            );
        }
        crate::FrameworkError::__NonExhaustive => panic!(),
    }

    Ok(())
}

/// An autocomplete function that can be used for the command parameter in your help function.
///
/// See `examples/framework_usage` for an example
pub async fn autocomplete_command<'a, U, E>(
    ctx: crate::Context<'a, U, E>,
    partial: &'a str,
) -> impl Iterator<Item = String> + 'a {
    ctx.framework()
        .options()
        .commands
        .iter()
        .filter(move |cmd| cmd.name.starts_with(partial))
        .map(|cmd| cmd.name.to_string())
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
    use std::fmt::Write as _;

    let mut show_private_guilds = false;
    if let crate::Context::Application(_) = ctx {
        if let Ok(app) = ctx.discord().http.get_current_application_info().await {
            if let Some(owner) = &app.owner {
                if owner.id == ctx.author().id {
                    show_private_guilds = true;
                }
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
    let mut num_unavailable_guilds = 0;
    let mut guilds = guild_ids
        .iter()
        .map(|&guild_id| {
            let guild = ctx.discord().cache.guild(guild_id)?;
            Some(Guild {
                name: guild.name.clone(),
                num_members: guild.member_count,
                is_public: guild.features.iter().any(|x| x == "DISCOVERABLE"),
            })
        })
        .filter_map(|guild| {
            if guild.is_none() {
                num_unavailable_guilds += 1;
            }
            guild
        })
        .collect::<Vec<_>>();
    guilds.sort_by_key(|guild| u64::MAX - guild.num_members); // descending sort

    let mut num_private_guilds = 0;
    let mut num_private_guild_members = 0;
    let mut response = format!("I am currently in {} servers!\n", guild_ids.len());
    for guild in guilds {
        if guild.is_public || show_private_guilds {
            let _ = writeln!(
                response,
                "- **{}** ({} members)",
                guild.name, guild.num_members
            );
        } else {
            num_private_guilds += 1;
            num_private_guild_members += guild.num_members;
        }
    }
    if num_private_guilds > 0 {
        let _ = writeln!(
            response,
            "- [{} private servers with {} members total]",
            num_private_guilds, num_private_guild_members
        );
    }
    if num_unavailable_guilds > 0 {
        let _ = writeln!(
            response,
            "- [{} unavailable servers (cache is not ready yet)]",
            num_unavailable_guilds
        );
    }

    if show_private_guilds {
        response += "\n_Showing private guilds because you are the bot owner_\n";
    }

    say_ephemeral(ctx, &response, show_private_guilds).await?;

    Ok(())
}
