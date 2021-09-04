use crate::serenity_prelude as serenity;

fn find_matching_slash_command<'a, U, E>(
    framework: &'a crate::Framework<U, E>,
    interaction: &serenity::ApplicationCommandInteractionData,
) -> Option<&'a crate::ApplicationCommand<U, E>> {
    let commands = &framework.options.application_options.commands;
    commands.iter().find(|cmd| match cmd {
        crate::ApplicationCommand::Slash(cmd) => {
            cmd.name == &interaction.name
                && interaction.kind == serenity::ApplicationCommandType::ChatInput
        }
        crate::ApplicationCommand::ContextMenu(cmd) => {
            let application_command_type = match &cmd.action {
                crate::ContextMenuCommandAction::User(_) => serenity::ApplicationCommandType::User,
                crate::ContextMenuCommandAction::Message(_) => {
                    serenity::ApplicationCommandType::Message
                }
            };
            cmd.name == &interaction.name && interaction.kind == application_command_type
        }
    })
}

pub async fn dispatch_interaction<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), (E, crate::ApplicationCommandErrorContext<'a, U, E>)> {
    let command = match find_matching_slash_command(this, &interaction.data) {
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

    // Only continue if command check returns true
    let command_check = command
        .options()
        .check
        .unwrap_or(this.options.application_options.command_check);
    let check_passes = command_check(ctx).await.map_err(|e| {
        (
            e,
            crate::ApplicationCommandErrorContext {
                command,
                ctx,
                while_checking: true,
            },
        )
    })?;
    if !check_passes {
        return Ok(());
    }

    if command
        .options()
        .defer_response
        .unwrap_or(this.options.application_options.defer_response)
    {
        if let Err(e) = ctx.defer_response().await {
            println!("Failed to send interaction acknowledgement: {}", e);
        }
    }

    (this.options.pre_command)(crate::Context::Application(ctx)).await;

    let action_result = match command {
        crate::ApplicationCommand::Slash(cmd) => (cmd.action)(ctx, &interaction.data.options).await,
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
