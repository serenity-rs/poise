use crate::PrefixCommand;

/// This file provides a bunch of utility functions like help menus or error handlers to use as a
/// starting point for the framework.

type BoxErrorSendSync = Box<dyn std::error::Error + Send + Sync>;

/// An error handler that prints the error into the console and also into the Discord chat.
/// If the user invoked the command wrong
/// (i.e. an [`crate::ArgumentParseError`]), the command help is displayed and the user is directed
/// to the help menu.
pub async fn on_error<D>(e: BoxErrorSendSync, ctx: crate::ErrorContext<'_, D, BoxErrorSendSync>) {
    println!("Encountered an error: {:?}", e);
    match ctx {
        crate::ErrorContext::Command(ctx) => {
            let user_error_msg = if let Some(crate::ArgumentParseError(e)) = e.downcast_ref() {
                // If we caught an argument parse error, give a helpful error message with the
                // command explanation if available

                let mut usage = "Please check the help menu for usage information".into();
                if let crate::CommandErrorContext::Prefix(ctx) = &ctx {
                    if let Some(multiline_help) = &ctx.command.options.multiline_help {
                        usage = multiline_help();
                    }
                }
                format!("**{}**\n{}", e, usage)
            } else {
                e.to_string()
            };
            if let Err(e) = crate::say_reply(ctx.ctx(), user_error_msg).await {
                println!("Error while user command error: {}", e);
            }
        }
        crate::ErrorContext::Listener(event) => {
            println!("Error in listener while processing {:?}: {}", event, e)
        }
        crate::ErrorContext::Setup => println!("Setup failed: {}", e),
    }
}

pub async fn help<D, E>(
    ctx: crate::Context<'_, D, E>,
    command: Option<&str>,
    extra_text_at_bottom: &str,
) -> Result<(), serenity::Error> {
    let reply = if let Some(command) = command {
        if let Some(command) = ctx
            .framework()
            .options()
            .prefix_options
            .commands
            .iter()
            .find(|cmd| cmd.name == command)
        {
            match command.options.multiline_help {
                Some(f) => f(),
                None => command
                    .options
                    .inline_help
                    .unwrap_or("No help available")
                    .to_owned(),
            }
        } else {
            format!("No such command `{}`", command)
        }
    } else {
        let is_also_a_slash_command = |command_name| {
            let slash_commands = &ctx.framework().options().slash_options.commands;
            slash_commands.iter().any(|c| c.name == command_name)
        };

        let mut categories: Vec<(Option<&str>, Vec<&PrefixCommand<_, _>>)> = Vec::new();
        for cmd in &ctx.framework().options().prefix_options.commands {
            if let Some((_, commands)) = categories
                .iter_mut()
                .find(|(key, _)| *key == cmd.options.category)
            {
                commands.push(cmd);
            } else {
                categories.push((cmd.options.category, vec![cmd]));
            }
        }

        let mut menu = String::from("```\n");
        for (category_name, commands) in categories {
            menu += category_name.unwrap_or("Commands");
            menu += ":\n";
            for command in commands {
                if command.options.hide_in_help {
                    continue;
                }

                let prefix = if is_also_a_slash_command(command.name) {
                    "/"
                } else {
                    ctx.framework().prefix()
                };

                menu += &format!(
                    "  {}{:<12}{}\n",
                    prefix,
                    command.name,
                    command.options.inline_help.unwrap_or("")
                );
            }
        }
        menu += "\n";
        menu += extra_text_at_bottom;
        menu += "\n```";

        menu
    };

    crate::say_reply(ctx, reply).await?;

    Ok(())
}
