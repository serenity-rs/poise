//! The cache variant of prefix permissions calculation

use crate::{serenity_prelude as serenity, PrefixContext};

use crate::dispatch::permissions::PermissionsInfo;

/// Gets the permissions of the ctx author and the bot.
pub(in crate::dispatch::permissions) async fn get_author_and_bot_permissions<U, E>(
    ctx: PrefixContext<'_, U, E>,
    guild_id: serenity::GuildId,
    skip_author: bool,
    skip_bot: bool,
) -> Option<PermissionsInfo>
where
    U: Send + Sync + 'static,
{
    let http = ctx.http();
    let guild = guild_id.to_partial_guild(http).await.ok()?;
    let guild_channel = {
        let channel = ctx.http().get_channel(ctx.channel_id()).await.ok()?;
        channel.guild().expect("channel should be a guild channel")
    };

    let bot_permissions = if skip_bot {
        None
    } else {
        let bot_member = guild.id.member(http, ctx.framework.bot_id).await.ok()?;
        Some(guild.user_permissions_in(&guild_channel, &bot_member))
    };

    let author_permissions = if skip_author {
        None
    } else {
        let err = "should always be Some in MessageCreateEvent";
        let author_member = ctx.msg.member.as_ref().expect(err);
        Some(guild.partial_member_permissions_in(&guild_channel, ctx.author().id, author_member))
    };

    Some(PermissionsInfo {
        author_permissions,
        bot_permissions,
    })
}
