//! Holds slash-command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture, Framework};

#[non_exhaustive]
pub struct SlashContext<'a, U, E> {
    pub discord: &'a serenity::Context,
    pub interaction: &'a serenity::ApplicationCommandInteraction,
    pub has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    pub framework: &'a Framework<U, E>,
    pub command: &'a SlashCommand<U, E>,
    pub data: &'a U,
}
impl<U, E> Clone for SlashContext<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U, E> Copy for SlashContext<'_, U, E> {}
impl<U, E> crate::_GetGenerics for SlashContext<'_, U, E> {
    type U = U;
    type E = E;
}

impl<U, E> SlashContext<'_, U, E> {
    pub async fn defer_response(&self) -> Result<(), serenity::Error> {
        self.interaction
            .create_interaction_response(self.discord, |f| {
                f.kind(serenity::InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await?;
        self.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

pub struct SlashCommandErrorContext<'a, U, E> {
    pub while_checking: bool,
    pub command: &'a SlashCommand<U, E>,
    pub ctx: SlashContext<'a, U, E>,
}

impl<U, E> Clone for SlashCommandErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        Self {
            while_checking: self.while_checking,
            command: self.command,
            ctx: self.ctx,
        }
    }
}

pub struct SlashCommandOptions<U, E> {
    /// Falls back to the framework-specified value on None. See there for documentation.
    pub on_error: Option<fn(E, SlashCommandErrorContext<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(SlashContext<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Falls back to the framework-specified value on None. See there for documentation.
    pub defer_response: Option<bool>,
    /// Whether responses to this command should be ephemeral by default.
    pub ephemeral: bool,
    /// Permissions which a user needs to have so that the slash command runs.
    pub required_permissions: serenity::Permissions,
    /// If true, only users from the [owners list](crate::FrameworkOptions::owners) may use this
    /// command.
    pub owners_only: bool,
}

impl<U, E> Default for SlashCommandOptions<U, E> {
    fn default() -> Self {
        Self {
            on_error: None,
            check: None,
            defer_response: None,
            ephemeral: false,
            required_permissions: serenity::Permissions::empty(),
            owners_only: false,
        }
    }
}

pub enum SlashCommandKind<U, E> {
    ChatInput {
        description: &'static str,
        parameters: Vec<
            fn(
                &mut serenity::CreateApplicationCommandOption,
            ) -> &mut serenity::CreateApplicationCommandOption,
        >,
        action: for<'a> fn(
            SlashContext<'a, U, E>,
            &'a [serenity::ApplicationCommandInteractionDataOption],
        ) -> BoxFuture<'a, Result<(), E>>,
    },
    User {
        action: fn(SlashContext<'_, U, E>, serenity::User) -> BoxFuture<'_, Result<(), E>>,
    },
    Message {
        action: fn(SlashContext<'_, U, E>, serenity::Message) -> BoxFuture<'_, Result<(), E>>,
    },
}

pub struct SlashCommand<U, E> {
    pub name: &'static str,
    pub options: SlashCommandOptions<U, E>,
    pub kind: SlashCommandKind<U, E>,
}

impl<U, E> SlashCommand<U, E> {
    pub fn create<'a>(
        &self,
        interaction: &'a mut serenity::CreateApplicationCommand,
    ) -> &'a mut serenity::CreateApplicationCommand {
        interaction.name(self.name);

        match &self.kind {
            SlashCommandKind::ChatInput {
                description,
                parameters,
                action: _,
            } => {
                interaction.description(description);
                for create_option in parameters {
                    let mut option = serenity::CreateApplicationCommandOption::default();
                    create_option(&mut option);
                    interaction.add_option(option);
                }
            }
            SlashCommandKind::User { action: _ } => {
                interaction.kind(serenity::ApplicationCommandType::User);
            }
            SlashCommandKind::Message { action: _ } => {
                interaction.kind(serenity::ApplicationCommandType::Message);
            }
        }

        interaction
    }
}

pub struct SlashFrameworkOptions<U, E> {
    /// List of bot commands.
    pub commands: Vec<SlashCommand<U, E>>,
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// Individual commands may override this callback.
    pub command_check: fn(SlashContext<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>,
    /// Whether to send an interaction acknoweldgement.
    ///
    /// Individual commands may override this value.
    ///
    /// If true, send back a quick interaction acknowledgement when receiving an interaction, which
    /// gives your code 15 minutes to respond. When this field is set to false, no acknowledgement
    /// is sent and you need to respond within 3 seconds or the interaction is considered failed by
    /// Discord.
    ///
    /// In some way this is the equivalent of `crate::PrefixFrameworkOptions::broadcast_typing`.
    pub defer_response: bool,
    /// Invoked when a user tries to execute a slash command but doesn't have the required
    /// permissions for it.
    ///
    /// This handler should be used to reply with some form of error message. If this handler does
    /// nothing, the user will be shown "Interaction failed" by their Discord client.
    pub missing_permissions_handler: fn(SlashContext<'_, U, E>) -> BoxFuture<'_, ()>,
}

impl<U: Send + Sync, E> Default for SlashFrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            command_check: |_| Box::pin(async { Ok(true) }),
            defer_response: false,
            missing_permissions_handler: |ctx| {
                Box::pin(async move {
                    let response = format!(
                        "You don't have the required permissions for `/{}`",
                        ctx.command.name
                    );
                    let _: Result<_, _> =
                        crate::send_slash_reply(ctx, |f| f.content(response).ephemeral(true)).await;
                })
            },
        }
    }
}
