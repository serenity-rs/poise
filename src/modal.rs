//! Modal trait and utility items for implementing it (mainly for the derive macro)

use crate::serenity_prelude as serenity;

/// Meant for use in derived [`Modal::parse`] implementation
///
/// _Takes_ the String out of the data. Logs warnings on unexpected state
#[doc(hidden)]
pub fn find_modal_text(
    data: &mut serenity::ModalSubmitInteractionData,
    custom_id: &str,
) -> Option<String> {
    for row in &mut data.components {
        let text = match row.components.get_mut(0) {
            Some(serenity::ActionRowComponent::InputText(text)) => text,
            Some(_) => {
                log::warn!("unexpected non input text component in modal response");
                continue;
            }
            None => {
                log::warn!("empty action row in modal response");
                continue;
            }
        };

        if text.custom_id == custom_id {
            let value = std::mem::take(&mut text.value);
            return if value.is_empty() { None } else { Some(value) };
        }
    }
    log::warn!(
        "{} not found in modal response (expected at least blank string)",
        custom_id
    );
    None
}

/// Derivable trait for modal interactions, Discords version of interactive forms
///
/// You don't need to implement this trait manually; use `#[derive(poise::Modal)]` instead
///
/// # Example
///
/// ```rust
/// # use poise::serenity_prelude as serenity;
/// # type Data = ();
/// # type Error = serenity::Error;
/// use poise::Modal;
/// type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;
///
/// #[derive(Debug, Modal)]
/// #[name = "Modal title"] // Struct name by default
/// struct MyModal {
///     #[name = "First input label"] // Field name by default
///     #[placeholder = "Your first input goes here"] // No placeholder by default
///     #[min_length = 5] // No length restriction by default (so, 1-4000 chars)
///     #[max_length = 500]
///     first_input: String,
///     #[name = "Second input label"]
///     #[paragraph] // Switches from single-line input to multiline text box
///     second_input: Option<String>, // Option means optional input
/// }
///
/// #[poise::command(slash_command)]
/// pub async fn modal(ctx: ApplicationContext<'_>) -> Result<(), Error> {
///     let data = MyModal::execute(ctx).await?;
///     println!("Got data: {:?}", data);
///
///     Ok(())
/// }
/// ```
#[async_trait::async_trait]
pub trait Modal: Sized {
    /// Returns an interaction response builder which creates the modal for this type
    fn create() -> serenity::CreateInteractionResponse;

    /// Parses a received modal submit interaction into this type
    ///
    /// Returns an error if a field was missing. This should never happen, because Discord will only
    /// let users submit when all required fields are filled properly
    fn parse(data: serenity::ModalSubmitInteractionData) -> Result<Self, &'static str>;

    /// Convenience function which:
    /// 1. sends the modal via [`Self::create()`]
    /// 2. waits for the user to submit via [`serenity::CollectModalInteraction`]
    /// 3. acknowledges the submitted data so that Discord closes the pop-up for the user
    /// 4. parses the submitted data via [`Self::parse()`], wrapping errors in [`serenity::Error::Other`]
    async fn execute<U: Send + Sync, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
    ) -> Result<Self, serenity::Error> {
        let interaction = ctx.interaction.unwrap();

        // Send modal
        interaction
            .create_interaction_response(ctx.discord, |b| {
                *b = Self::create();
                b
            })
            .await?;
        ctx.has_sent_initial_response
            .store(true, std::sync::atomic::Ordering::SeqCst);

        // Wait for user to submit
        let response = serenity::CollectModalInteraction::new(&ctx.discord.shard)
            .author_id(interaction.user.id)
            .await
            .unwrap();

        // Send acknowledgement so that the pop-up is closed
        response
            .create_interaction_response(ctx.discord, |b| {
                b.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
            })
            .await?;

        Ok(Self::parse(response.data.clone()).map_err(serenity::Error::Other)?)
    }
}
