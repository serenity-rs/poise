//! Dispatches interactions onto framework commands

use crate::serenity_prelude as serenity;

/// Check if the interaction with the given name and arguments matches any framework command
fn find_matching_command<'a, 'b, U, E>(
    interaction_name: &str,
    interaction_options: &'b [serenity::ResolvedOption<'b>],
    commands: &'a [crate::Command<U, E>],
) -> Option<(&'a crate::Command<U, E>, &'b [serenity::ResolvedOption<'b>])> {
    commands.iter().find_map(|cmd| {
        if interaction_name != cmd.name && Some(interaction_name) != cmd.context_menu_name {
            return None;
        }

        if let Some((sub_interaction_name, sub_interaction_options)) = interaction_options
            .iter()
            .find_map(|option| match &option.value {
                serenity::ResolvedValue::SubCommand(o)
                | serenity::ResolvedValue::SubCommandGroup(o) => Some((&option.name, o)),
                _ => None,
            })
        {
            find_matching_command(
                sub_interaction_name,
                sub_interaction_options,
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
    // need to pass the following in for lifetime reasons
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    options: &'a [serenity::ResolvedOption<'a>],
) -> Result<
    crate::ApplicationContext<'a, U, E>,
    Option<(crate::FrameworkError<'a, U, E>, &'a crate::Command<U, E>)>,
> {
    let search_result = find_matching_command(
        &interaction.data().name,
        options,
        &framework.options.commands,
    );
    let (command, leaf_interaction_options) = search_result.ok_or_else(|| {
        log::warn!(
            "received unknown interaction \"{}\"",
            interaction.data().name
        );
        None
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

    super::common::check_permissions_and_cooldown(ctx.into(), command)
        .await
        .map_err(|e| Some((e, command)))?;

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
    // Need to pass this in from outside because of lifetime issues
    options: &'a [serenity::ResolvedOption<'a>],
) -> Result<(), Option<(crate::FrameworkError<'a, U, E>, &'a crate::Command<U, E>)>> {
    let ctx = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(interaction),
        has_sent_initial_response,
        invocation_data,
        options,
    )
    .await?;

    (framework.options.pre_command)(crate::Context::Application(ctx)).await;

    // Check which interaction type we received and grab the command action and, if context menu,
    // the resolved click target, and execute the action
    let command_structure_mismatch_error = Some((
        crate::FrameworkError::CommandStructureMismatch {
            ctx,
            description: "received interaction type but command contained no \
                matching action or interaction contained no matching context menu object",
        },
        ctx.command,
    ));
    let action_result = match interaction.data.kind {
        serenity::CommandType::ChatInput => {
            let action = ctx
                .command
                .slash_action
                .ok_or(command_structure_mismatch_error)?;
            action(ctx).await
        }
        serenity::CommandType::User => {
            match (ctx.command.context_menu_action, interaction.data.target()) {
                (
                    Some(crate::ContextMenuCommandAction::User(action)),
                    Some(serenity::ResolvedTarget::User(user, _)),
                ) => action(ctx, user.clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        serenity::CommandType::Message => {
            match (ctx.command.context_menu_action, interaction.data.target()) {
                (
                    Some(crate::ContextMenuCommandAction::Message(action)),
                    Some(serenity::ResolvedTarget::Message(message)),
                ) => action(ctx, message.clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        _ => return Err(None),
    };
    action_result.map_err(|e| Some((e, ctx.command)))?;

    (framework.options.post_command)(crate::Context::Application(ctx)).await;

    Ok(())
}

/// Dispatches this interaction onto framework commands, i.e. runs the associated autocomplete
/// callback
pub async fn dispatch_autocomplete<'a, U, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    // Need to pass this in from outside because of lifetime issues
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    // Need to pass this in from outside because of lifetime issues
    options: &'a [serenity::ResolvedOption<'a>],
) -> Result<(), Option<(crate::FrameworkError<'a, U, E>, &'a crate::Command<U, E>)>> {
    let ctx = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(interaction),
        has_sent_initial_response,
        invocation_data,
        options,
    )
    .await?;

    // Find which parameter is focused by the user
    let (focused_option_name, partial_input) = ctx
        .args
        .iter()
        .find_map(|o| match &o.value {
            serenity::ResolvedValue::Autocomplete { value, .. } => Some((&o.name, value)),
            _ => None,
        })
        .ok_or(None)?;

    // Find the matching parameter from our Command object
    let parameters = &ctx.command.parameters;
    let focused_parameter = parameters
        .iter()
        .find(|p| &p.name == focused_option_name)
        .ok_or(None)?;

    // If this parameter supports autocomplete...
    if let Some(autocomplete_callback) = focused_parameter.autocomplete_callback {
        #[allow(unused_imports)]
        use ::serenity::json::prelude::*; // as_str() access via trait for simd-json

        // Generate an autocomplete response
        let autocomplete_response = match autocomplete_callback(ctx, partial_input).await {
            Ok(x) => x,
            Err(e) => {
                log::warn!("couldn't generate autocomplete response: {}", e);
                return Err(None);
            }
        };

        // Send the generates autocomplete response
        if let Err(e) = interaction
            .create_autocomplete_response(&ctx.discord.http, autocomplete_response)
            .await
        {
            log::warn!("couldn't send autocomplete response: {}", e);
        }
    }

    Ok(())
}
