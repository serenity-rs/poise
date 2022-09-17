//! Dispatches interactions onto framework commands

use crate::serenity_prelude as serenity;

/// Check if the interaction with the given name and arguments matches any framework command
fn find_matching_command<'a, 'b, U, E>(
    interaction_name: &str,
    interaction_options: &'b [serenity::CommandDataOption],
    commands: &'a [crate::Command<U, E>],
) -> Option<(&'a crate::Command<U, E>, &'b [serenity::CommandDataOption])> {
    commands.iter().find_map(|cmd| {
        if interaction_name != cmd.name && Some(interaction_name) != cmd.context_menu_name {
            return None;
        }

        if let Some(sub_interaction) = interaction_options.iter().find(|option| {
            option.kind == serenity::CommandOptionType::SubCommand
                || option.kind == serenity::CommandOptionType::SubCommandGroup
        }) {
            find_matching_command(
                &sub_interaction.name,
                &sub_interaction.options,
                &cmd.subcommands,
            )
        } else {
            Some((cmd, interaction_options))
        }
    })
}

/// Given an interaction, finds the matching framework command and checks if the user is allowed
/// access
pub async fn extract_command_and_run_checks<'a, U, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    ctx: &'a serenity::Context,
    interaction: crate::ApplicationCommandOrAutocompleteInteraction<'a>,
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
) -> Result<crate::ApplicationContext<'a, U, E>, crate::FrameworkError<'a, U, E>> {
    let search_result = find_matching_command(
        &interaction.data().name,
        &interaction.data().options,
        &framework.options.commands,
    );
    let (command, leaf_interaction_options) =
        search_result.ok_or(crate::FrameworkError::UnknownInteraction {
            ctx,
            framework,
            interaction,
        })?;

    let ctx = crate::ApplicationContext {
        data: framework.user_data().await,
        discord: ctx,
        framework,
        interaction,
        args: leaf_interaction_options,
        command,
        has_sent_initial_response,
        invocation_data,
        __non_exhaustive: (),
    };

    super::common::check_permissions_and_cooldown(ctx.into()).await?;

    Ok(ctx)
}

/// Dispatches this interaction onto framework commands, i.e. runs the associated command
pub async fn dispatch_interaction<'a, U, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    // Need to pass this in from outside because of lifetime issues
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    let ctx = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(interaction),
        has_sent_initial_response,
        invocation_data,
    )
    .await?;

    (framework.options.pre_command)(crate::Context::Application(ctx)).await;

    // Check which interaction type we received and grab the command action and, if context menu,
    // the resolved click target, and execute the action
    let command_structure_mismatch_error = crate::FrameworkError::CommandStructureMismatch {
        ctx,
        description: "received interaction type but command contained no \
                matching action or interaction contained no matching context menu object",
    };
    let action_result = match interaction.data.kind {
        serenity::CommandType::ChatInput => {
            let action = ctx
                .command
                .slash_action
                .ok_or(command_structure_mismatch_error)?;
            action(ctx).await
        }
        serenity::CommandType::User => {
            match (ctx.command.context_menu_action, &interaction.data.target()) {
                (
                    Some(crate::ContextMenuCommandAction::User(action)),
                    Some(serenity::ResolvedTarget::User(user, _)),
                ) => action(ctx, user.clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        serenity::CommandType::Message => {
            match (ctx.command.context_menu_action, &interaction.data.target()) {
                (
                    Some(crate::ContextMenuCommandAction::Message(action)),
                    Some(serenity::ResolvedTarget::Message(message)),
                ) => action(ctx, *message.clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        other => {
            log::warn!("unknown interaction command type: {:?}", other);
            return Ok(());
        }
    };
    action_result?;

    (framework.options.post_command)(crate::Context::Application(ctx)).await;

    Ok(())
}

/// Dispatches this interaction onto framework commands, i.e. runs the associated autocomplete
/// callback
pub async fn dispatch_autocomplete<'a, U, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::AutocompleteInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    // Need to pass this in from outside because of lifetime issues
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    let ctx = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(interaction),
        has_sent_initial_response,
        invocation_data,
    )
    .await?;

    // Find which parameter is focused by the user
    let focused_option = match ctx.args.iter().find(|o| o.focused) {
        Some(x) => x,
        None => {
            log::warn!("no option is focused in autocomplete interaction");
            return Ok(());
        }
    };

    // Find the matching parameter from our Command object
    let parameters = &ctx.command.parameters;
    let focused_parameter = parameters
        .iter()
        .find(|p| p.name == focused_option.name)
        .ok_or(crate::FrameworkError::CommandStructureMismatch {
            ctx,
            description: "focused autocomplete parameter name not recognized",
        })?;

    // Only continue if this parameter supports autocomplete and Discord has given us a partial value
    let (autocomplete_callback, partial_input) = match (
        focused_parameter.autocomplete_callback,
        &focused_option.value,
    ) {
        (Some(a), Some(b)) => (a, b),
        _ => return Ok(()),
    };

    #[allow(unused_imports)]
    use ::serenity::json::prelude::*; // as_str() access via trait for simd-json

    // Generate an autocomplete response
    let partial_input =
        partial_input
            .as_str()
            .ok_or(crate::FrameworkError::CommandStructureMismatch {
                ctx,
                description: "unexpected non-string autocomplete input",
            })?;
    let autocomplete_response = match autocomplete_callback(ctx, partial_input).await {
        Ok(x) => x,
        Err(e) => {
            log::warn!("couldn't generate autocomplete response: {}", e);
            return Ok(());
        }
    };

    // Send the generates autocomplete response
    if let Err(e) = interaction
        .create_autocomplete_response(&ctx.discord.http, |b| {
            *b = autocomplete_response;
            b
        })
        .await
    {
        log::warn!("couldn't send autocomplete response: {}", e);
    }

    Ok(())
}
