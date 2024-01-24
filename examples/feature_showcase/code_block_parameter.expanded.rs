mod code_block_parameter {
    use crate::{Context, Error};
    #[allow(clippy::str_to_string)]
    pub fn code() -> ::poise::Command<
        <Context<'static> as poise::_GetGenerics>::U,
        <Context<'static> as poise::_GetGenerics>::E,
    > {
        pub async fn inner(
            ctx: Context<'_>,
            args: poise::KeyValueArgs,
            code: poise::CodeBlock,
        ) -> Result<(), Error> {
            ctx.say({
                    let res = ::alloc::fmt::format(
                        format_args!("Key value args: {0:?}\nCode: {1}", args, code),
                    );
                    res
                })
                .await?;
            Ok(())
        }
        ::poise::Command {
            prefix_action: Some(|ctx| Box::pin(async move {
                let (poise_param_0, poise_param_1, ..) = async {
                    use ::poise::PopArgument as _;
                    let ctx = ctx.serenity_context;
                    let msg = ctx.msg;
                    let args = ctx.args;
                    let attachment_index = 0;
                    let mut error: (
                        Box<dyn std::error::Error + Send + Sync>,
                        Option<String>,
                    ) = (
                        Box::new(::poise::TooManyArguments {
                            __non_exhaustive: (),
                        }) as _,
                        None,
                    );
                    match {
                        use ::poise::PopArgumentHack as _;
                        (&std::marker::PhantomData::<poise::KeyValueArgs>)
                            .pop_from(&args, attachment_index, ctx, msg)
                    }
                        .await
                    {
                        Ok((args, attachment_index, token)) => {
                            match {
                                use ::poise::PopArgumentHack as _;
                                (&std::marker::PhantomData::<poise::CodeBlock>)
                                    .pop_from(&args, attachment_index, ctx, msg)
                            }
                                .await
                            {
                                Ok((args, attachment_index, token)) => {
                                    if args.is_empty() {
                                        return Ok((token, token));
                                    }
                                }
                                Err(e) => error = e,
                            };
                        }
                        Err(e) => error = e,
                    };
                    Err(error)
                }
                    .await
                    .map_err(|(error, input)| poise::FrameworkError::new_argument_parse(
                        ctx.into(),
                        input,
                        error,
                    ))?;
                if !ctx.framework.options.manual_cooldowns {
                    ctx.command
                        .cooldowns
                        .lock()
                        .unwrap()
                        .start_cooldown(ctx.cooldown_context());
                }
                inner(ctx.into(), poise_param_0, poise_param_1)
                    .await
                    .map_err(|error| poise::FrameworkError::new_command(
                        ctx.into(),
                        error,
                    ))
            })),
            slash_action: None,
            context_menu_action: None,
            subcommands: ::alloc::vec::Vec::new(),
            subcommand_required: false,
            name: "code".to_string(),
            name_localizations: std::collections::HashMap::new(),
            qualified_name: String::from("code"),
            identifying_name: String::from("code"),
            source_code_name: String::from("code"),
            category: None,
            description: None,
            description_localizations: std::collections::HashMap::new(),
            help_text: None,
            hide_in_help: false,
            manual_cooldowns: (/*ERROR*/),
            cooldowns: std::sync::Mutex::new(::poise::Cooldowns::new()),
            cooldown_config: std::sync::RwLock::default(),
            reuse_response: false,
            default_member_permissions: poise::serenity_prelude::Permissions::empty(),
            required_permissions: poise::serenity_prelude::Permissions::empty(),
            required_bot_permissions: poise::serenity_prelude::Permissions::empty(),
            owners_only: false,
            guild_only: false,
            dm_only: false,
            nsfw_only: false,
            checks: ::alloc::vec::Vec::new(),
            on_error: None,
            parameters: <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    ::poise::CommandParameter {
                        name: "args".to_string(),
                        name_localizations: std::collections::HashMap::new(),
                        description: None,
                        description_localizations: std::collections::HashMap::new(),
                        required: true,
                        channel_types: None,
                        type_setter: None,
                        choices: ::alloc::vec::Vec::new(),
                        autocomplete_callback: None,
                        __non_exhaustive: (),
                    },
                    ::poise::CommandParameter {
                        name: "code".to_string(),
                        name_localizations: std::collections::HashMap::new(),
                        description: None,
                        description_localizations: std::collections::HashMap::new(),
                        required: true,
                        channel_types: None,
                        type_setter: None,
                        choices: ::alloc::vec::Vec::new(),
                        autocomplete_callback: None,
                        __non_exhaustive: (),
                    },
                ]),
            ),
            custom_data: Box::new(()),
            aliases: ::alloc::vec::Vec::new(),
            invoke_on_edit: false,
            track_deletion: false,
            broadcast_typing: false,
            context_menu_name: None,
            ephemeral: false,
            __non_exhaustive: (),
        }
    }
}
