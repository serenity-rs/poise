//! Application command permissions calculation
use crate::serenity_prelude as serenity;

use super::PermissionsInfo;

/// Gets the permissions of the ctx author and the bot.
pub(super) fn get_author_and_bot_permissions(
    interaction: &serenity::CommandInteraction,
) -> PermissionsInfo {
    let err = "member is Some if interaction is in guild";
    let author_member = interaction.member.as_ref().expect(err);

    let err = "should always be some as inside interaction";
    let author_permissions = author_member.permissions.expect(err);

    let err = "should always be some according to discord docs";
    let bot_permissions = interaction.app_permissions.expect(err);

    PermissionsInfo {
        author_permissions: Some(author_permissions),
        bot_permissions: Some(bot_permissions),
    }
}
