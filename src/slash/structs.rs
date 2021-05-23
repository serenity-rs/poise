//! Holds slash-command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture, Framework};

pub struct SlashContext<'a, U, E> {
    pub discord: &'a serenity::Context,
    pub interaction: &'a serenity::Interaction,
    pub has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    pub framework: &'a Framework<U, E>,
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
    /// Fall back to the framework-specified value on None.
    pub on_error: Option<fn(E, SlashCommandErrorContext<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(SlashContext<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Fall back to the framework-specified value on None.
    pub defer_response: Option<bool>,
}

impl<U, E> Default for SlashCommandOptions<U, E> {
    fn default() -> Self {
        Self {
            on_error: None,
            check: None,
            defer_response: None,
        }
    }
}

pub struct SlashCommand<U, E> {
    pub name: &'static str,
    pub description: &'static str,
    pub action: for<'a> fn(
        SlashContext<'a, U, E>,
        &'a [serenity::ApplicationCommandInteractionDataOption],
    ) -> BoxFuture<'a, Result<(), E>>,
    pub parameters: Vec<
        fn(
            &mut serenity::CreateApplicationCommandOption,
        ) -> &mut serenity::CreateApplicationCommandOption,
    >,
    pub options: SlashCommandOptions<U, E>,
}

impl<U, E> SlashCommand<U, E> {
    pub async fn create_in_guild(
        &self,
        http: &serenity::Http,
        guild_id: serenity::GuildId,
    ) -> Result<serenity::ApplicationCommand, serenity::Error> {
        guild_id
            .create_application_command(http, |c| self.create(c))
            .await
    }

    pub async fn create_global(
        &self,
        http: &serenity::Http,
    ) -> Result<serenity::ApplicationCommand, serenity::Error> {
        serenity::ApplicationCommand::create_global_application_command(http, |c| self.create(c))
            .await
    }

    pub fn create<'a>(
        &self,
        interaction: &'a mut serenity::CreateApplicationCommand,
    ) -> &'a mut serenity::CreateApplicationCommand {
        interaction.name(self.name);
        interaction.description(self.description);
        for create_option in &self.parameters {
            let mut option = serenity::CreateApplicationCommandOption::default();
            create_option(&mut option);
            interaction.add_option(option);
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
}

impl<U, E> Default for SlashFrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            command_check: |_| Box::pin(async { Ok(true) }),
            defer_response: false,
        }
    }
}
