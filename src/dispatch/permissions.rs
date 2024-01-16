//! Module for calculating permission checks for commands

use crate::serenity_prelude as serenity;

mod application;
mod prefix;

/// Simple POD type to hold the results of permission lookups.
struct PermissionsInfo {
    /// The Permissions of the author, if requested.
    author_permissions: Option<serenity::Permissions>,
    /// The Permissions of the bot, if requested.
    bot_permissions: Option<serenity::Permissions>,
}

impl PermissionsInfo {
    /// Returns the fake permissions info from a DM.
    fn dm_permissions() -> Self {
        Self {
            author_permissions: Some(serenity::Permissions::dm_permissions()),
            bot_permissions: Some(serenity::Permissions::dm_permissions()),
        }
    }
}

/// Retrieves the permissions for the context author and the bot.
async fn get_author_and_bot_permissions<U, E>(
    ctx: crate::Context<'_, U, E>,
    skip_author: bool,
    skip_bot: bool,
) -> Option<PermissionsInfo>
where
    U: Send + Sync + 'static,
{
    // No permission checks in DMs.
    let Some(guild_id) = ctx.guild_id() else {
        return Some(PermissionsInfo::dm_permissions());
    };

    match ctx {
        crate::Context::Application(ctx) => {
            Some(application::get_author_and_bot_permissions(ctx.interaction))
        }
        crate::Context::Prefix(ctx) => {
            prefix::get_author_and_bot_permissions(ctx, guild_id, skip_author, skip_bot).await
        }
    }
}

/// Retrieves the set of permissions that are lacking, relative to the given required permission set
///
/// Returns None if permissions couldn't be retrieved.
pub(super) async fn calculate_missing<U, E>(
    ctx: crate::Context<'_, U, E>,
    author_required_permissions: serenity::Permissions,
    bot_required_permissions: serenity::Permissions,
) -> Option<(serenity::Permissions, serenity::Permissions)>
where
    U: Send + Sync + 'static,
{
    // If both user and bot are None, return empty permissions
    if author_required_permissions.is_empty() && bot_required_permissions.is_empty() {
        return Some((
            serenity::Permissions::empty(),
            serenity::Permissions::empty(),
        ));
    }

    // Fetch permissions, returning None if an error occurred
    let permissions = get_author_and_bot_permissions(
        ctx,
        author_required_permissions.is_empty(),
        bot_required_permissions.is_empty(),
    )
    .await?;

    let author_missing_perms = permissions
        .author_permissions
        .map(|permissions| author_required_permissions - permissions)
        .unwrap_or_default();

    let bot_missing_perms = permissions
        .bot_permissions
        .map(|permissions| bot_required_permissions - permissions)
        .unwrap_or_default();

    Some((author_missing_perms, bot_missing_perms))
}
