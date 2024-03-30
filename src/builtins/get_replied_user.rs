//! Contains a command utility that can be used to retrieve a replied user.

/// Retrieves the user from a message reply or gets the initial user (the one selected by a ping or
/// in the slash command interaction).
pub fn get_replied_user<U, E>(
    ctx: crate::Context<'_, U, E>,
) -> Option<crate::serenity::model::user::User> {
    let crate::Context::Prefix(msg_ctx) = ctx else {
        return None;
    };
    let ref_msg = msg_ctx.msg.referenced_message.as_deref();
    ref_msg.map(|x| x.author.clone())
}
