//! Building blocks for common commands like help commands or application command registration
//!
//! This file provides sample commands and utility functions like help menus or error handlers to
//! use as a starting point for the framework.

mod help;
pub use help::*;

mod register;
pub use register::*;

#[cfg(any(feature = "chrono", feature = "time"))]
mod paginate;
#[cfg(any(feature = "chrono", feature = "time"))]
pub use paginate::*;

use crate::serenity_prelude as serenity;

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
            eprintln!("Error in user data setup: {}", error);
        }
        crate::FrameworkError::EventHandler { error, event, .. } => log::error!(
            "User event event handler encountered an error on {} event: {}",
            event.name(),
            error
        ),
        crate::FrameworkError::Command { ctx, error } => {
            let error = error.to_string();
            eprintln!("An error occured in a command: {}", error);
            ctx.say(error).await?;
        }
        crate::FrameworkError::SubcommandRequired { ctx } => {
            let subcommands = ctx
                .command()
                .subcommands
                .iter()
                .map(|s| &*s.name)
                .collect::<Vec<_>>();
            let response = format!(
                "You must specify one of the following subcommands: {}",
                subcommands.join(", ")
            );
            ctx.send(|b| b.content(response).ephemeral(true)).await?;
        }
        crate::FrameworkError::CommandPanic { ctx, payload: _ } => {
            // Not showing the payload to the user because it may contain sensitive info
            ctx.send(|b| {
                b.embed(|b| {
                    b.title("Internal error")
                        .color((255, 0, 0))
                        .description("An unexpected internal error has occurred.")
                })
                .ephemeral(true)
            })
            .await?;
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
        crate::FrameworkError::__NonExhaustive(unreachable) => match unreachable {},
    }

    Ok(())
}

/// An autocomplete function that can be used for the command parameter in your help function.
///
/// See `examples/feature_showcase` for an example
#[allow(clippy::unused_async)] // Required for the return type
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

    let show_private_guilds = ctx.framework().options().owners.contains(&ctx.author().id);

    // Aggregate all guilds and sort them by size
    let mut hidden_guilds = 0;
    let mut hidden_guilds_members = 0;
    let mut shown_guilds = Vec::<(String, u64)>::new();
    for guild_id in ctx.cache().guilds() {
        match ctx.cache().guild_field(guild_id, |g| {
            (
                g.name.clone(),
                g.member_count,
                g.features.iter().any(|x| x == "DISCOVERABLE"),
            )
        }) {
            Some((name, member_count, is_public)) => {
                if !is_public && !show_private_guilds {
                    hidden_guilds += 1; // private guild whose name and size shouldn't be exposed
                } else {
                    shown_guilds.push((name, member_count))
                }
            }
            None => hidden_guilds += 1, // uncached guild
        }
    }
    shown_guilds.sort_by_key(|(_, member)| u64::MAX - member); // sort largest guilds first

    // Iterate guilds and build up the response message line by line
    let mut response = format!(
        "I am currently in {} servers!\n",
        shown_guilds.len() + hidden_guilds
    );
    if show_private_guilds {
        response.insert_str(0, "_Showing private guilds because you are a bot owner_\n");
    }
    let mut guilds = shown_guilds.into_iter().peekable();
    while let Some((name, member_count)) = guilds.peek() {
        let line = format!("- **{}** ({} members)\n", name, member_count);

        // Make sure we don't exceed a certain number of characters below the 2000 char limit so
        // we have enough space for the remaining servers line
        if response.len() + line.len() > 1940 {
            for (_remaining_guild_name, members) in guilds {
                hidden_guilds += 1;
                hidden_guilds_members += members;
            }
            break;
        }

        response += &line;
        guilds.next(); // advance peekable iterator
    }
    if hidden_guilds > 0 {
        let _ = writeln!(
            response,
            "- {} remaining servers with {} members total",
            hidden_guilds, hidden_guilds_members
        );
    }

    // Final safe guard (shouldn't be hit at the time of writing)
    if response.len() > 2000 {
        let mut truncate_at = 2000;
        while !response.is_char_boundary(truncate_at) {
            truncate_at -= 1;
        }
        response.truncate(truncate_at);
    }

    // If we show sensitive data (private guilds), it mustn't be made public, so it's ephemeral
    ctx.send(|b| b.content(response).ephemeral(show_private_guilds))
        .await?;

    Ok(())
}
