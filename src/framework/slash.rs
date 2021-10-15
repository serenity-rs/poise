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

pub async fn dispatch_interaction<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), (E, crate::ApplicationCommandErrorContext<'a, U, E>)> {
    let (command, leaf_interaction_options) =
        match find_matching_application_command(this, &interaction.data) {
            Some(value) => value,
            None => {
                println!(
                    "Warning: received unknown interaction \"{}\"",
                    interaction.data.name
                );
                return Ok(());
            }
        };

    let ctx = crate::ApplicationContext {
        data: this.get_user_data().await,
        discord: ctx,
        framework: this,
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
        (this.options.application_options.missing_permissions_handler)(ctx).await;
        return Ok(());
    }

    // Only continue if command checks returns true
    let checks_passing = (|| async {
        let global_check_passes = match &this.options.command_check {
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
                command,
                ctx,
                while_checking: true,
            },
        )
    })?;
    if !checks_passing {
        return Ok(());
    }

    (this.options.pre_command)(crate::Context::Application(ctx)).await;

    let action_result = match command {
        crate::ApplicationCommand::Slash(cmd) => (cmd.action)(ctx, leaf_interaction_options).await,
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

    action_result.map_err(|e| {
        (
            e,
            crate::ApplicationCommandErrorContext {
                command,
                ctx,
                while_checking: false,
            },
        )
    })
}
