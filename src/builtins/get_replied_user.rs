//! Contains a command utility that can be used to retrieve a replied user.
//! By default

/// Optional configuration for the way the replied user is retrieved.
pub struct RepliedUserConfiguration {
    /// Whether the specified user will get returned in the case of no replied message.
    pub default_to_none: bool,
}

impl Default for RepliedUserConfiguration {
    fn default() -> Self {
        Self {
            default_to_none: true,
        }
    }
}

/// Retrieves the user from a message reply or gets the initial user (the one selected by a ping or
/// in the slash command interaction).
pub fn get_replied_user<U, E>(
    ctx: crate::Context<'_, U, E>,
    user: Option<crate::serenity::model::user::User>,
    config: RepliedUserConfiguration,
) -> Option<crate::serenity::model::user::User> {
    let selected_user = user?;
    let default_return = match config.default_to_none {
        true => Some(selected_user),
        false => None,
    };

    let crate::Context::Prefix(msg_ctx) = ctx else {
        return default_return;
    };

    let ref_ctx_msg = msg_ctx.msg.referenced_message.as_deref();

    match ref_ctx_msg {
        Some(ref_msg) => Some(ref_msg.author.clone()),
        None => default_return,
    }
}
