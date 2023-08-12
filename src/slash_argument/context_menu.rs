//! Contains a simple trait, implemented for all context menu command compatible parameter types
use crate::serenity_prelude as serenity;
use crate::BoxFuture;

/// Implemented for all types that can be used in a context menu command
pub trait ContextMenuParameter<U, E> {
    /// Convert an action function pointer that takes Self as an argument into the appropriate
    /// [`crate::ContextMenuCommandAction`] variant.
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, U, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ) -> crate::ContextMenuCommandAction<U, E>;
}

impl<U, E> ContextMenuParameter<U, E> for serenity::User {
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, U, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ) -> crate::ContextMenuCommandAction<U, E> {
        crate::ContextMenuCommandAction::User(action)
    }
}

impl<U, E> ContextMenuParameter<U, E> for serenity::Message {
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, U, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ) -> crate::ContextMenuCommandAction<U, E> {
        crate::ContextMenuCommandAction::Message(action)
    }
}
