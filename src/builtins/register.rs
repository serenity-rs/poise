//! Utilities for registering application commands

use crate::serenity_prelude as serenity;

/// Collects all commands into a `Vec<serenity::CreateApplicationCommand>`, which can be used
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
/// serenity::Command::set_global_application_commands(ctx.discord(), create_commands).await?;
/// # Ok(()) }
/// ```
pub fn create_application_commands<U, E>(
    commands: &[crate::Command<U, E>],
) -> Vec<serenity::CreateCommand> {
    /// We decided to extract context menu commands recursively, despite the subcommand hierarchy
    /// not being preserved. Because it's more confusing to just silently discard context menu
    /// commands if they're not top-level commands.
    /// https://discord.com/channels/381880193251409931/919310428344029265/947970605985189989
    fn recursively_add_context_menu_commands<U, E>(
        builder: &mut Vec<serenity::CreateCommand>,
        command: &crate::Command<U, E>,
    ) {
        if let Some(context_menu_command) = command.create_as_context_menu_command() {
            builder.push(context_menu_command);
        }
        for subcommand in &command.subcommands {
            recursively_add_context_menu_commands(builder, subcommand);
        }
    }

    let mut commands_builder = Vec::new();
    for command in commands {
        if let Some(slash_command) = command.create_as_slash_command() {
            commands_builder.push(slash_command);
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
    let num_commands = commands_builder.len();

    if global {
        ctx.say(format!("Registering {} commands...", num_commands))
            .await?;
        serenity::Command::set_global_application_commands(ctx.discord(), commands_builder).await?;
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
            .set_application_commands(ctx.discord(), commands_builder)
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
    let num_commands = create_commands.len();

    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    let reply = ctx
        .send(
            crate::CreateReply::default()
                .content("Choose what to do with the commands:")
                .components(
                    serenity::CreateComponents::default()
                        .add_action_row(
                            serenity::CreateActionRow::default()
                                .add_button(
                                    serenity::CreateButton::new(
                                        serenity::ButtonStyle::Primary,
                                        "register.guild",
                                    )
                                    .label("Register in guild"),
                                )
                                .add_button(
                                    serenity::CreateButton::new(
                                        serenity::ButtonStyle::Danger,
                                        "unregister.guild",
                                    )
                                    .label("Delete in guild"),
                                ),
                        )
                        .add_action_row(
                            serenity::CreateActionRow::default()
                                .add_button(
                                    serenity::CreateButton::new(
                                        serenity::ButtonStyle::Primary,
                                        "register.global",
                                    )
                                    .label("Register globally"),
                                )
                                .add_button(
                                    serenity::CreateButton::new(
                                        serenity::ButtonStyle::Danger,
                                        "unregister.global",
                                    )
                                    .label("Delete globally"),
                                ),
                        ),
                ),
        )
        .await?;

    let interaction = reply
        .message()
        .await?
        .component_interaction_collector(&ctx.discord().shard)
        .author_id(ctx.author().id)
        .collect_single()
        .await;

    // remove buttons after button press
    reply
        .edit(
            ctx,
            crate::CreateReply::default()
                .components(serenity::builder::CreateComponents::default()),
        )
        .await?;

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
            ctx.say(format!("Registering {} global commands...", num_commands))
                .await?;
            serenity::Command::set_global_application_commands(ctx.discord(), create_commands)
                .await?;
        } else {
            ctx.say("Unregistering global commands...").await?;
            serenity::Command::set_global_application_commands(ctx.discord(), Vec::new()).await?;
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
            ctx.say(format!("Registering {} guild commands...", num_commands))
                .await?;
            guild_id
                .set_application_commands(ctx.discord(), create_commands)
                .await?;
        } else {
            ctx.say("Unregistering guild commands...").await?;
            guild_id
                .set_application_commands(ctx.discord(), Vec::new())
                .await?;
        }
    }

    ctx.say("Done!").await?;
    Ok(())
}
