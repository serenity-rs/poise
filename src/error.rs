/// Contains the location of the error with location-specific context
pub enum ErrorContext<'a, U, E> {
    Listener(&'a crate::Event<'a>),
    Command(CommandErrorContext<'a, U, E>),
}

impl<U, E> Clone for ErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Listener(x) => Self::Listener(x),
            Self::Command(x) => Self::Command(x.clone()),
        }
    }
}

pub struct CommandErrorContext<'a, U, E> {
    pub while_checking: bool,
    pub command: &'a crate::Command<U, E>,
    pub ctx: crate::Context<'a, U, E>,
}

impl<U, E> Clone for CommandErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        Self {
            while_checking: self.while_checking,
            command: self.command,
            ctx: self.ctx,
        }
    }
}
