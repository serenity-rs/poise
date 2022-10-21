//! Utilities for registering application commands

use crate::serenity_prelude as serenity;

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
/// serenity::Command::set_global_application_commands(ctx.discord(), |b| {
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
    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    let commands_builder = create_application_commands(&ctx.framework().options().commands);
    let num_commands = commands_builder.0.len();

    if global {
        ctx.say(format!("Registering {} commands...", num_commands))
            .await?;
        serenity::Command::set_global_application_commands(ctx.discord(), |b| {
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

        ctx.say(format!("Registering {} commands...", num_commands))
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
///
/// ![Screenshot of output](https://imgur.com/rTbTaDs.png)
///
/// You probably want to use this by wrapping it in a small `register` command:
/// ```rust
/// # type Error = Box<dyn std::error::Error + Send + Sync>;
/// # type Context<'a> = poise::Context<'a, (), Error>;
/// #[poise::command(prefix_command)]
/// pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
///     poise::builtins::register_application_commands_buttons(ctx).await?;
///     Ok(())
/// }
///
/// // ...
/// poise::FrameworkOptions {
///     commands: vec![
///         // ...
///         register(),
///     ],
/// #   ..Default::default()
/// };
/// ```
///
/// Which you can call like any prefix command, for example `@your_bot register`.
pub async fn register_application_commands_buttons<U, E>(
    ctx: crate::Context<'_, U, E>,
) -> Result<(), serenity::Error> {
    let create_commands = create_application_commands(&ctx.framework().options().commands);
    let num_commands = create_commands.0.len();

    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    let reply = ctx
        .send(|m| {
            m.content("Choose what to do with the commands:")
                .components(|c| {
                    c.create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("register.guild")
                                .label("Register in guild")
                                .style(serenity::ButtonStyle::Primary)
                                .emoji(serenity::ReactionType::Unicode("📋".to_string()))
                        })
                        .create_button(|b| {
                            b.custom_id("unregister.guild")
                                .label("Delete in guild")
                                .style(serenity::ButtonStyle::Danger)
                                .emoji(serenity::ReactionType::Unicode("🗑️".to_string()))
                        })
                    })
                    .create_action_row(|r| {
                        r.create_button(|b| {
                            b.custom_id("register.global")
                                .label("Register globally")
                                .style(serenity::ButtonStyle::Primary)
                                .emoji(serenity::ReactionType::Unicode("📋".to_string()))
                        })
                        .create_button(|b| {
                            b.custom_id("unregister.global")
                                .label("Delete globally")
                                .style(serenity::ButtonStyle::Danger)
                                .emoji(serenity::ReactionType::Unicode("🗑️".to_string()))
                        })
                    })
                })
        })
        .await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx.discord())
        .author_id(ctx.author().id)
        .await;

    reply.edit(ctx, |b| b.components(|b| b)).await?; // remove buttons after button press
    // NOTE: Can this be done in one .edit?
    reply.edit(ctx, |b| b.content("Processing... Please wait.")).await?; // Edit message to processing after button press 
    let pressed_button_id = match &interaction {
        Some(m) => &m.data.custom_id,
        None => {
            ctx.say(":warning: You didn't interact in time - please run the command again.").await?;
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
            ctx.say(format!(":gear: Registering {} global commands...", num_commands))
                .await?;
            serenity::Command::set_global_application_commands(ctx.discord(), |b| {
                *b = create_commands;
                b
            })
            .await?;
        } else {
            ctx.say(":gear: Unregistering global commands...").await?;
            serenity::Command::set_global_application_commands(ctx.discord(), |b| b).await?;
        }
    } else {
        let guild_id = match ctx.guild_id() {
            Some(x) => x,
            None => {
                ctx.say(":x: Must be called in guild").await?;
                return Ok(());
            }
        };
        if register {
            ctx.say(format!(":gear: Registering {} guild commands...", num_commands))
                .await?;
            guild_id
                .set_application_commands(ctx.discord(), |b| {
                    *b = create_commands;
                    b
                })
                .await?;
        } else {
            ctx.say(":gear: Unregistering guild commands...").await?;
            guild_id
                .set_application_commands(ctx.discord(), |b| b)
                .await?;
        }
    }

    ctx.say(":white_check_mark: Done!").await?;
    Ok(())
}
