use crate::serenity_prelude as serenity;

pub async fn dispatch_interaction<'a, U, E>(
    this: &'a super::Framework<U, E>,
    ctx: &'a serenity::Context,
    interaction: &'a serenity::ApplicationCommandInteraction,
    name: &'a str,
    options: &'a [serenity::ApplicationCommandInteractionDataOption],
    // Need to pass this in from outside because of lifetime issues
    has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
) -> Result<(), (E, crate::SlashCommandErrorContext<'a, U, E>)> {
    let command = match this
        .options
        .slash_options
        .commands
        .iter()
        .find(|cmd| cmd.name == name)
    {
        Some(x) => x,
        None => {
            println!("Warning: received unknown interaction \"{}\"", name);
            return Ok(());
        }
    };

    let ctx = crate::SlashContext {
        data: this.get_user_data().await,
        discord: ctx,
        framework: this,
        interaction,
        command,
        has_sent_initial_response,
    };

    // Make sure that user has required permissions
    if !super::check_user_permissions(
        crate::Context::Slash(ctx),
        command.options.required_permissions,
    )
    .await
    {
        (this.options.slash_options.missing_permissions_handler)(ctx).await;
        return Ok(());
    }

    // Only continue if command check returns true
    let command_check = command
        .options
        .check
        .unwrap_or(this.options.slash_options.command_check);
    let check_passes = command_check(ctx).await.map_err(|e| {
        (
            e,
            crate::SlashCommandErrorContext {
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
        .options
        .defer_response
        .unwrap_or(this.options.slash_options.defer_response)
    {
        if let Err(e) = ctx.defer_response().await {
            println!("Failed to send interaction acknowledgement: {}", e);
        }
    }

    (this.options.pre_command)(crate::Context::Slash(ctx)).await;

    (command.action)(ctx, options).await.map_err(|e| {
        (
            e,
            crate::SlashCommandErrorContext {
                command,
                ctx,
                while_checking: false,
            },
        )
    })
}
