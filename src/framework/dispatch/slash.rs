use crate::serenity_prelude as serenity;

/// Small utility function to create the scuffed ad-hoc error type we use in this module
fn make_error<'a, U, E>(
    error: E,
    ctx: crate::ApplicationContext<'a, U, E>,
    location: crate::CommandErrorLocation,
) -> (crate::FrameworkError<'a, U, E>, &'a crate::CommandId<U, E>) {
    let error = crate::FrameworkError::Command {
        error,
        ctx: crate::Context::Application(ctx),
        location,
    };
    (error, ctx.command.id())
}

fn find_matching_application_command<'a, 'b, U, E>(
    framework: &'a crate::Framework<U, E>,
    interaction: &'b serenity::ApplicationCommandInteractionData,
) -> Option<(
    crate::ApplicationCommand<'a, U, E>,
    &'b [serenity::ApplicationCommandInteractionDataOption],
)> {
    let commands = &framework.options.application_options.commands;
    commands.iter().find_map(|cmd| match cmd {
        crate::ApplicationCommandTree::ContextMenu(cmd) => {
            let application_command_type = match &cmd.action {
                crate::ContextMenuCommandAction::User(_) => serenity::ApplicationCommandType::User,
                crate::ContextMenuCommandAction::Message(_) => {
                    serenity::ApplicationCommandType::Message
                }
            };
            if cmd.name == interaction.name && interaction.kind == application_command_type {
                Some((
                    crate::ApplicationCommand::ContextMenu(cmd),
                    &*interaction.options,
                ))
            } else {
                None
            }
        }
        // TODO: improve this monstrosity
        crate::ApplicationCommandTree::Slash(cmd_meta) => match cmd_meta {
            crate::SlashCommandMeta::Command(cmd) => {
                if cmd.name == interaction.name
                    && interaction.kind == serenity::ApplicationCommandType::ChatInput
                {
                    Some((crate::ApplicationCommand::Slash(cmd), &*interaction.options))
                } else {
                    None
                }
            }
            // TODO: check name field perhaps?
            crate::SlashCommandMeta::CommandGroup {
                subcommands,
                name: _,
                description: _,
                id: _,
            } => {
                if cmd_meta.name() != interaction.name {
                    return None;
                }
                let interaction = match interaction.options.iter().find(|option| {
                    option.kind == serenity::ApplicationCommandOptionType::SubCommand
                        || option.kind == serenity::ApplicationCommandOptionType::SubCommandGroup
                }) {
                    Some(x) => x,
                    None => {
                        eprintln!("Expected slash subcommand, but Discord didn't send one");
                        return None;
                    }
                };

                subcommands.iter().find_map(|cmd| match cmd {
                    crate::SlashCommandMeta::Command(cmd) => {
                        if cmd.name == interaction.name {
                            Some((crate::ApplicationCommand::Slash(cmd), &*interaction.options))
                        } else {
                            None
                        }
                    }
                    // TODO: check name field perhaps?
                    crate::SlashCommandMeta::CommandGroup {
                        subcommands,
                        name: _,
                        description: _,
                        id: _,
                    } => {
                        let interaction = match interaction.options.iter().find(|option| {
                            option.kind == serenity::ApplicationCommandOptionType::SubCommand
                                || option.kind
                                    == serenity::ApplicationCommandOptionType::SubCommandGroup
                        }) {
                            Some(x) => x,
                            None => {
                                eprintln!("Expected slash subcommand, but Discord didn't send one");
                                return None;
                            }
                        };

                        subcommands.iter().find_map(|cmd| match cmd {
                            crate::SlashCommandMeta::Command(cmd) => {
                                if cmd.name == interaction.name {
                                    Some((
                                        crate::ApplicationCommand::Slash(cmd),
                                        &*interaction.options,
                                    ))
                                } else {
                                    None
                                }
                            }
                            crate::SlashCommandMeta::CommandGroup { .. } => {
                                // Discord doesn't send nested slash commands at this level anymore
                                None
                            }
                        })
                    }
                })
            }
        },
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
    Option<(crate::FrameworkError<'a, U, E>, &'a crate::CommandId<U, E>)>,
> {
    let (command, leaf_interaction_options) =
        find_matching_application_command(framework, interaction.data()).ok_or_else(|| {
            println!(
                "Warning: received unknown interaction \"{}\"",
                interaction.data().name
            );
            None
        })?;

    let ctx = crate::ApplicationContext {
        data: framework.get_user_data().await,
        discord: ctx,
        framework,
        interaction,
        command,
        has_sent_initial_response,
    };

    super::common::check_permissions_and_cooldown(ctx.into(), command.id())
        .await
        .map_err(|e| Some((e, &**command.id())))?;

    Ok((ctx, leaf_interaction_options))
}

pub async fn dispatch_interaction<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), Option<(crate::FrameworkError<'a, U, E>, &'a crate::CommandId<U, E>)>> {
    let (ctx, options) = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(interaction),
        has_sent_initial_response,
    )
    .await?;

    (framework.options.pre_command)(crate::Context::Application(ctx)).await;

    let action_result = match ctx.command {
        crate::ApplicationCommand::Slash(cmd) => (cmd.action)(ctx, options).await,
        crate::ApplicationCommand::ContextMenu(cmd) => match cmd.action {
            crate::ContextMenuCommandAction::User(action) => match &interaction.data.target {
                Some(serenity::ResolvedTarget::User(user, _)) => (action)(ctx, user.clone()).await,
                _ => {
                    println!("Warning: no user object sent in user context menu interaction");
                    return Ok(());
                }
            },
            crate::ContextMenuCommandAction::Message(action) => match &interaction.data.target {
                Some(serenity::ResolvedTarget::Message(msg)) => (action)(ctx, msg.clone()).await,
                _ => {
                    println!("Warning: no message object sent in message context menu interaction");
                    return Ok(());
                }
            },
        },
    };

    (framework.options.post_command)(crate::Context::Application(ctx)).await;

    action_result.map_err(|e| Some(make_error(e, ctx, crate::CommandErrorLocation::Body)))
}

pub async fn dispatch_autocomplete<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::AutocompleteInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), Option<(crate::FrameworkError<'a, U, E>, &'a crate::CommandId<U, E>)>> {
    let (ctx, options) = extract_command_and_run_checks(
        framework,
        ctx,
        crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(interaction),
        has_sent_initial_response,
    )
    .await?;

    let command = match ctx.command {
        crate::ApplicationCommand::Slash(x) => x,
        crate::ApplicationCommand::ContextMenu(_) => return Err(None),
    };

    for param in &command.parameters {
        let autocomplete_callback = match param.autocomplete_callback {
            Some(x) => x,
            None => continue,
        };

        if let Err(error) = autocomplete_callback(ctx, interaction, options).await {
            let command = &ctx.command;
            command.id().on_error.unwrap_or(framework.options.on_error)(
                crate::FrameworkError::Command {
                    ctx: crate::Context::Application(ctx),
                    error,
                    location: crate::CommandErrorLocation::Autocomplete,
                },
            )
            .await;
        }
    }

    Ok(())
}
