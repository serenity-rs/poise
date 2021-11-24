use crate::serenity_prelude as serenity;

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
        crate::ApplicationCommandTree::Slash(cmd) => match cmd {
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
            } => {
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
    framework: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: crate::ApplicationCommandOrAutocompleteInteraction<'a>,
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<
    (
        crate::ApplicationContext<'a, U, E>,
        &'a [serenity::ApplicationCommandInteractionDataOption],
    ),
    Option<(E, crate::ApplicationCommandErrorContext<'a, U, E>)>,
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

    // Make sure that user has required permissions
    if !super::check_required_permissions_and_owners_only(
        crate::Context::Application(ctx),
        command.options().required_permissions,
        command.options().owners_only,
    )
    .await
    {
        (framework
            .options
            .application_options
            .missing_permissions_handler)(ctx)
        .await;
        return Err(None);
    }

    // Only continue if command checks returns true
    let checks_passing = (|| async {
        let global_check_passes = match &framework.options.command_check {
            Some(check) => check(crate::Context::Application(ctx)).await?,
            None => true,
        };

        let command_specific_check_passes = match &command.options().check {
            Some(check) => check(ctx).await?,
            None => true,
        };

        Ok(global_check_passes && command_specific_check_passes)
    })()
    .await
    .map_err(|e| {
        (
            e,
            crate::ApplicationCommandErrorContext {
                ctx,
                location: crate::CommandErrorLocation::Check,
            },
        )
    })?;
    if !checks_passing {
        return Err(None);
    }

    let cooldowns = &command.id().cooldowns;
    let cooldown_left = cooldowns.lock().unwrap().get_wait_time(ctx.into());
    if let Some(cooldown_left) = cooldown_left {
        if let Some(callback) = ctx.framework.options().cooldown_hit {
            callback(ctx.into(), cooldown_left).await.map_err(|e| {
                Some((
                    e,
                    crate::ApplicationCommandErrorContext {
                        ctx,
                        location: crate::CommandErrorLocation::CooldownCallback,
                    },
                ))
            })?;
        }
        return Err(None);
    }
    cooldowns.lock().unwrap().start_cooldown(ctx.into());

    Ok((ctx, leaf_interaction_options))
}

pub async fn dispatch_interaction<'a, U, E>(
    framework: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), Option<(E, crate::ApplicationCommandErrorContext<'a, U, E>)>> {
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

    action_result.map_err(|e| {
        Some((
            e,
            crate::ApplicationCommandErrorContext {
                ctx,
                location: crate::CommandErrorLocation::Body,
            },
        ))
    })
}

pub async fn dispatch_autocomplete<'a, U, E>(
    framework: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::AutocompleteInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), Option<(E, crate::ApplicationCommandErrorContext<'a, U, E>)>> {
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

        if let Err(e) = autocomplete_callback(ctx, interaction, options).await {
            let error_ctx = crate::ApplicationCommandErrorContext {
                ctx,
                location: crate::CommandErrorLocation::Autocomplete,
            };

            if let Some(on_error) = error_ctx.ctx.command.options().on_error {
                on_error(e, error_ctx).await;
            } else {
                (framework.options.on_error)(e, crate::ErrorContext::Autocomplete(error_ctx)).await;
            }
        }
    }

    Ok(())
}
