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

pub fn user_info() -> (
    poise::PrefixCommand<Data, Error>,
    Option<poise::SlashCommand<Data, Error>>,
) {
    let prefix_cmd = poise::PrefixCommand {
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
    };
    let slash_cmd = poise::SlashCommand {
        name: "User information",
        kind: poise::SlashCommandKind::User {
            action: |ctx, user| {
                Box::pin(async move { user_info_inner(Context::Slash(ctx), &user).await })
            },
        },
        options: poise::SlashCommandOptions::default(),
    };

    (prefix_cmd, Some(slash_cmd))
}
