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
    // Should only fail if the guild is not cached, which is fair to bail on.
    let guild = ctx.cache().guild(guild_id)?;

    let author_permissions = if skip_author {
        None
    } else {
        Some(ctx.msg.author_permissions(ctx.cache())?)
    };

    let bot_permissions = if skip_bot {
        None
    } else {
        let channel_id = ctx.channel_id();
        let bot_user_id = ctx.framework.bot_id();
        Some(get_bot_permissions(&guild, channel_id, bot_user_id)?)
    };

    Some(PermissionsInfo {
        author_permissions,
        bot_permissions,
    })
}

/// Gets the permissions for the bot.
fn get_bot_permissions(
    guild: &serenity::Guild,
    channel_id: serenity::ChannelId,
    bot_id: serenity::UserId,
) -> Option<serenity::Permissions> {
    // Should never fail, as the bot member is always cached
    let bot_member = guild.members.get(&bot_id)?;

    if let Some(channel) = guild.channels.get(&channel_id) {
        Some(guild.user_permissions_in(channel, bot_member))
    } else if let Some(thread) = guild.threads.iter().find(|th| th.id == channel_id) {
        let err = "parent id should always be Some for thread";
        let parent_channel_id = thread.parent_id.expect(err);

        let parent_channel = guild.channels.get(&parent_channel_id)?;
        let mut parent_permissions = guild.user_permissions_in(parent_channel, bot_member);

        parent_permissions.set(
            serenity::Permissions::SEND_MESSAGES,
            parent_permissions.send_messages_in_threads(),
        );

        Some(parent_permissions)
    } else {
        // The message was either:
        // - Sent in a guild with broken caching
        // - Not set in a channel or thread?
        tracing::warn!("Could not find channel/thread ({channel_id}) for permissions check in cache for guild: {}", guild.id);
        None
    }
}
