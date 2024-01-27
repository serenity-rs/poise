//! Dispatches interactions onto framework commands

use crate::serenity_prelude as serenity;

/// Check if the interaction with the given name and arguments matches any framework command
fn find_matching_command<'a, 'b, U, E>(
    interaction_name: &str,
    interaction_options: &'b [serenity::ResolvedOption<'b>],
    commands: &'a [crate::Command<U, E>],
    parent_commands: &mut Vec<&'a crate::Command<U, E>>,
) -> Option<(&'a crate::Command<U, E>, &'b [serenity::ResolvedOption<'b>])> {
    commands.iter().find_map(|cmd| {
        if interaction_name != cmd.name
            && Some(interaction_name) != cmd.context_menu_name.as_deref()
        {
            return None;
        }

        if let Some((sub_name, sub_interaction)) =
            interaction_options
                .iter()
                .find_map(|option| match &option.value {
                    serenity::ResolvedValue::SubCommand(o)
                    | serenity::ResolvedValue::SubCommandGroup(o) => Some((&option.name, o)),
                    _ => None,
                })
        {
            parent_commands.push(cmd);
            find_matching_command(sub_name, sub_interaction, &cmd.subcommands, parent_commands)
        } else {
            Some((cmd, interaction_options))
        }
    })
}

/// Parses an `Interaction` into a [`crate::ApplicationContext`] using some context data.
///
/// After this, the [`crate::ApplicationContext`] should be passed into [`run_command`] or
/// [`run_autocomplete`].
#[allow(clippy::too_many_arguments)] // We need to pass them all in to create Context.
fn extract_command<'a, U, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    interaction: &'a serenity::CommandInteraction,
    interaction_type: crate::CommandInteractionType,
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    options: &'a [serenity::ResolvedOption<'a>],
    parent_commands: &'a mut Vec<&'a crate::Command<U, E>>,
) -> Result<crate::ApplicationContext<'a, U, E>, crate::FrameworkError<'a, U, E>> {
    let search_result = find_matching_command(
        &interaction.data.name,
        options,
        &framework.options.commands,
        parent_commands,
    );
    let (command, leaf_interaction_options) =
        search_result.ok_or(crate::FrameworkError::UnknownInteraction {
            framework,
            interaction,
        })?;

    Ok(crate::ApplicationContext {
        framework,
        interaction,
        interaction_type,
        args: leaf_interaction_options,
        command,
        parent_commands,
        has_sent_initial_response,
        invocation_data,
        __non_exhaustive: (),
    })
}

/// Given an interaction, finds the matching framework command and checks if the user is allowed access
#[allow(clippy::too_many_arguments)] // We need to pass them all in to create Context.
pub async fn extract_command_and_run_checks<'a, U: Send + Sync + 'static, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    interaction: &'a serenity::CommandInteraction,
    interaction_type: crate::CommandInteractionType,
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    options: &'a [serenity::ResolvedOption<'a>],
    parent_commands: &'a mut Vec<&'a crate::Command<U, E>>,
) -> Result<crate::ApplicationContext<'a, U, E>, crate::FrameworkError<'a, U, E>> {
    let ctx = extract_command(
        framework,
        interaction,
        interaction_type,
        has_sent_initial_response,
        invocation_data,
        options,
        parent_commands,
    )?;
    super::common::check_permissions_and_cooldown(ctx.into()).await?;
    Ok(ctx)
}

/// Given the extracted application command data from [`extract_command`], runs the command,
/// including all the before and after code like checks.
async fn run_command<U: Send + Sync + 'static, E>(
    ctx: crate::ApplicationContext<'_, U, E>,
) -> Result<(), crate::FrameworkError<'_, U, E>> {
    super::common::check_permissions_and_cooldown(ctx.into()).await?;

    (ctx.framework.options.pre_command)(crate::Context::Application(ctx)).await;

    // Check which interaction type we received and grab the command action and, if context menu,
    // the resolved click target, and execute the action
    let command_structure_mismatch_error = crate::FrameworkError::CommandStructureMismatch {
        ctx,
        description: "received interaction type but command contained no \
                matching action or interaction contained no matching context menu object",
    };
    let action_result = match ctx.interaction.data.kind {
        serenity::CommandType::ChatInput => {
            let action = ctx
                .command
                .slash_action
                .ok_or(command_structure_mismatch_error)?;
            action(ctx).await
        }
        serenity::CommandType::User => {
            match (
                ctx.command.context_menu_action,
                &ctx.interaction.data.target(),
            ) {
                (
                    Some(crate::ContextMenuCommandAction::User(action)),
                    Some(serenity::ResolvedTarget::User(user, _)),
                ) => action(ctx, (*user).clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        serenity::CommandType::Message => {
            match (
                ctx.command.context_menu_action,
                &ctx.interaction.data.target(),
            ) {
                (
                    Some(crate::ContextMenuCommandAction::Message(action)),
                    Some(serenity::ResolvedTarget::Message(message)),
                ) => action(ctx, (*message).clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        other => {
            tracing::warn!("unknown interaction command type: {:?}", other);
            return Ok(());
        }
    };
    action_result?;

    (ctx.framework.options.post_command)(crate::Context::Application(ctx)).await;

    Ok(())
}

/// Dispatches this interaction onto framework commands, i.e. runs the associated command
pub async fn dispatch_interaction<'a, U: Send + Sync + 'static, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    interaction: &'a serenity::CommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    // Need to pass this in from outside because of lifetime issues
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    // Need to pass this in from outside because of lifetime issues
    options: &'a [serenity::ResolvedOption<'a>],
    parent_commands: &'a mut Vec<&'a crate::Command<U, E>>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    let ctx = extract_command(
        framework,
        interaction,
        crate::CommandInteractionType::Command,
        has_sent_initial_response,
        invocation_data,
        options,
        parent_commands,
    )?;

    crate::catch_unwind_maybe(run_command(ctx))
        .await
        .map_err(|payload| crate::FrameworkError::CommandPanic {
            payload,
            ctx: ctx.into(),
        })??;

    Ok(())
}

/// Given the extracted application command data from [`extract_command`], runs the autocomplete
/// callbacks, including all the before and after code like checks.
async fn run_autocomplete<U: Send + Sync + 'static, E>(
    ctx: crate::ApplicationContext<'_, U, E>,
) -> Result<(), crate::FrameworkError<'_, U, E>> {
    super::common::check_permissions_and_cooldown(ctx.into()).await?;

    // Find which parameter is focused by the user
    let (focused_option_name, partial_input) = match ctx.args.iter().find_map(|o| match &o.value {
        serenity::ResolvedValue::Autocomplete { value, .. } => Some((&o.name, value)),
        _ => None,
    }) {
        Some(x) => x,
        None => {
            tracing::warn!("no option is focused in autocomplete interaction");
            return Ok(());
        }
    };

    // Find the matching parameter from our Command object
    let parameters = &ctx.command.parameters;
    let focused_parameter = parameters
        .iter()
        .find(|p| &p.name == focused_option_name)
        .ok_or(crate::FrameworkError::CommandStructureMismatch {
            ctx,
            description: "focused autocomplete parameter name not recognized",
        })?;

    // Only continue if this parameter supports autocomplete and Discord has given us a partial value
    let autocomplete_callback = match focused_parameter.autocomplete_callback {
        Some(a) => a,
        _ => return Ok(()),
    };

    // Generate an autocomplete response
    let autocomplete_response = autocomplete_callback(ctx, partial_input).await;

    // Send the generates autocomplete response
    if let Err(e) = ctx
        .interaction
        .create_response(
            ctx.http(),
            serenity::CreateInteractionResponse::Autocomplete(autocomplete_response),
        )
        .await
    {
        tracing::warn!("couldn't send autocomplete response: {e}");
    }

    Ok(())
}

/// Dispatches this interaction onto framework commands, i.e. runs the associated autocomplete
/// callback
pub async fn dispatch_autocomplete<'a, U: Send + Sync + 'static, E>(
    framework: crate::FrameworkContext<'a, U, E>,
    interaction: &'a serenity::CommandInteraction,
    // Need to pass the following in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    options: &'a [serenity::ResolvedOption<'a>],
    parent_commands: &'a mut Vec<&'a crate::Command<U, E>>,
) -> Result<(), crate::FrameworkError<'a, U, E>> {
    let ctx = extract_command(
        framework,
        interaction,
        crate::CommandInteractionType::Autocomplete,
        has_sent_initial_response,
        invocation_data,
        options,
        parent_commands,
    )?;

    crate::catch_unwind_maybe(run_autocomplete(ctx))
        .await
        .map_err(|payload| crate::FrameworkError::CommandPanic {
            payload,
            ctx: ctx.into(),
        })??;

    Ok(())
}
