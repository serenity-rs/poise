use crate::serenity_prelude as serenity;

fn find_matching_command<'a, 'b, U, E>(
    interaction_name: &str,
    interaction_options: &'b [serenity::ApplicationCommandInteractionDataOption],
    commands: &'a [crate::Command<U, E>],
) -> Option<(
    &'a crate::Command<U, E>,
    &'b [serenity::ApplicationCommandInteractionDataOption],
)> {
    commands.iter().find_map(|cmd| {
        if interaction_name != cmd.name && Some(interaction_name) != cmd.context_menu_name {
            return None;
        }

        if let Some(sub_interaction) = interaction_options.iter().find(|option| {
            option.kind == serenity::ApplicationCommandOptionType::SubCommand
                || option.kind == serenity::ApplicationCommandOptionType::SubCommandGroup
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

pub async fn extract_command_and_run_checks<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: crate::ApplicationCommandOrAutocompleteInteraction<'a>,
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<
    (
        crate::ApplicationContext<'a, U, E>,
        &'a [serenity::ApplicationCommandInteractionDataOption],
    ),
    Option<(crate::FrameworkError<'a, U, E>, &'a crate::Command<U, E>)>,
> {
    let search_result = find_matching_command(
        &interaction.data().name,
        &interaction.data().options,
        &framework.options.commands,
    );
    let (command, leaf_interaction_options) = search_result.ok_or_else(|| {
        println!(
            "Warning: received unknown interaction \"{}\"",
            interaction.data().name
        );
        None
    })?;

    let ctx = crate::ApplicationContext {
        data: framework.user_data().await,
        discord: ctx,
        framework,
        interaction,
        command,
        has_sent_initial_response,
    };

    super::common::check_permissions_and_cooldown(ctx.into(), command)
        .await
        .map_err(|e| Some((e, command)))?;

    Ok((ctx, leaf_interaction_options))
}

pub async fn dispatch_interaction<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), Option<(crate::FrameworkError<'a, U, E>, &'a crate::Command<U, E>)>> {
    let (ctx, options) = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(interaction),
        has_sent_initial_response,
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
        serenity::ApplicationCommandType::ChatInput => {
            let action = ctx
                .command
                .slash_action
                .ok_or(command_structure_mismatch_error)?;
            action(ctx, options).await
        }
        serenity::ApplicationCommandType::User => {
            match (ctx.command.context_menu_action, &interaction.data.target) {
                (
                    Some(crate::ContextMenuCommandAction::User(action)),
                    Some(serenity::ResolvedTarget::User(user, _)),
                ) => action(ctx, user.clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        serenity::ApplicationCommandType::Message => {
            match (ctx.command.context_menu_action, &interaction.data.target) {
                (
                    Some(crate::ContextMenuCommandAction::Message(action)),
                    Some(serenity::ResolvedTarget::Message(message)),
                ) => action(ctx, message.clone()).await,
                _ => return Err(command_structure_mismatch_error),
            }
        }
        _ => return Err(None),
    };

    (framework.options.post_command)(crate::Context::Application(ctx)).await;

    action_result.map_err(|e| Some((e, ctx.command)))
}

pub async fn dispatch_autocomplete<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::AutocompleteInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), Option<(crate::FrameworkError<'a, U, E>, &'a crate::Command<U, E>)>> {
    let (ctx, options) = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(interaction),
        has_sent_initial_response,
    )
    .await?;

    // Find which parameter is focused by the user
    let focused_option = options.iter().find(|o| o.focused).ok_or(None)?;

    // Find the matching parameter from our Command object
    let parameters = &ctx.command.parameters;
    let focused_parameter = parameters
        .iter()
        .find(|p| p.name == focused_option.name)
        .ok_or(None)?;

    // If this parameter supports autocomplete...
    if let Some(autocomplete_callback) = focused_parameter.autocomplete_callback {
        // Generate an autocomplete response
        let focused_option_json = focused_option.value.as_ref().ok_or(None)?;
        let autocomplete_response = match autocomplete_callback(ctx, focused_option_json).await {
            Ok(x) => x,
            Err(e) => {
                println!("Warning: couldn't generate autocomplete response: {}", e);
                return Err(None);
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
            println!("Warning: couldn't send autocomplete response: {}", e);
        }
    }

    Ok(())
}
