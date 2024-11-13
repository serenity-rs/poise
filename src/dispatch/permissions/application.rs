//! Application command permissions calculation
use crate::serenity_prelude as serenity;

use super::PermissionsInfo;

/// Gets the permissions of the ctx author and the bot.
pub(super) fn get_author_and_bot_permissions(
    interaction: &serenity::CommandInteraction,
) -> PermissionsInfo {
    let expect_err = "member is Some if interaction is in guild";
    let author_member = interaction.member.as_ref().expect(expect_err);

    PermissionsInfo {
        author_permissions: author_member.permissions,
        bot_permissions: interaction.app_permissions,
    }
}
