use crate::{Context, Data, Error};
use poise::serenity_prelude as serenity;

pub async fn user_info_inner(ctx: Context<'_>, user: &serenity::User) -> Result<(), Error> {
    let response = format!(
        "**Name**: {}\n**Created**: {}",
        user.name,
        user.created_at()
    );

    poise::say_reply(ctx, response).await?;
    Ok(())
}

pub fn user_info() -> poise::CommandDefinition<Data, Error> {
    poise::CommandDefinition {
        prefix: poise::PrefixCommand {
            name: "userinfo",
            action: |ctx, args| {
                Box::pin(async move {
                    let (member,) =
                        poise::parse_prefix_args!(ctx.discord, ctx.msg, args => (serenity::Member))
                            .await?;
                    user_info_inner(Context::Prefix(ctx), &member.user).await
                })
            },
            options: poise::PrefixCommandOptions::default(),
        },
        slash: Some(poise::SlashCommand {
            kind: poise::SlashCommandKind::ChatInput {
                name: "userinfo",
                description: "Query information about a Discord profile",
                action: |ctx, args| {
                    Box::pin(async move {
                        // ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: $($type:tt)*) => {
                        let (member,) = poise::parse_slash_args!(
                            &ctx.discord, ctx.interaction.guild_id, ctx.interaction.channel_id, args
                            => (user: serenity::Member)
                        )
                        .await?;
                        user_info_inner(Context::Slash(ctx), &member.user).await
                    })
                },
                parameters: vec![|f| {
                    f.kind(serenity::ApplicationCommandOptionType::User)
                        .name("user")
                        .description("Discord profile to query information about")
                        .required(true)
                }],
            },
            options: poise::SlashCommandOptions::default(),
        }),
        context_menu: Some(poise::SlashCommand {
            kind: poise::SlashCommandKind::User {
                name: "User information",
                action: |ctx, user| {
                    Box::pin(async move { user_info_inner(Context::Slash(ctx), &user).await })
                },
            },
            options: poise::SlashCommandOptions::default(),
        }),
    }
}
