//! Application command permissions calculation
use self::serenity::Permissions;
use super::PermissionsInfo;
use crate::serenity_prelude as serenity;

/// Gets the permissions of the ctx author and the bot.
pub(super) fn get_author_and_bot_permissions(
    interaction: &serenity::CommandInteraction,
) -> PermissionsInfo {
    let err = "member is Some if interaction is in guild";
    let author_member = interaction.member.as_ref().expect(err);

    let err = "should always be some as inside interaction";
    let mut author_permissions = author_member.permissions.expect(err);

    let mut bot_permissions = interaction.app_permissions;

    let channel = &interaction.channel;
    if matches!(channel, serenity::GenericInteractionChannel::Thread(_)) {
        author_permissions
            .set(Permissions::SEND_MESSAGES, author_permissions.send_messages_in_threads());
        bot_permissions.set(Permissions::SEND_MESSAGES, bot_permissions.send_messages_in_threads());
    }

    PermissionsInfo {
        author_permissions: Some(author_permissions),
        bot_permissions: Some(bot_permissions),
    }
}
