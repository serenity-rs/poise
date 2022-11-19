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

/// Convenience function for showing the modal and waiting for a response.
///
/// If the user doesn't submit before the timeout expires, `None` is returned.
///
/// Note: a modal must be the first response to a command. You cannot send any messages before,
/// or the modal will fail.
///
/// This function:
/// 1. sends the modal via [`Modal::create()`]
/// 2. waits for the user to submit via [`serenity::CollectModalInteraction`]
/// 3. acknowledges the submitted data so that Discord closes the pop-up for the user
/// 4. parses the submitted data via [`Modal::parse()`], wrapping errors in [`serenity::Error::Other`]
///
/// If you need more specialized behavior, you can copy paste the implementation of this function
/// and adjust to your needs. The code of this function is just a starting point.
pub async fn execute_modal<U: Send + Sync, E, M: Modal>(
    ctx: crate::ApplicationContext<'_, U, E>,
    defaults: Option<M>,
    timeout: Option<std::time::Duration>,
) -> Result<Option<M>, serenity::Error> {
    let interaction = ctx.interaction.unwrap();
    let interaction_id = interaction.id.to_string();

    // Send modal
    interaction
        .create_interaction_response(ctx.serenity_context, |b| {
            *b = M::create(defaults, interaction_id.clone());
            b
        })
        .await?;
    ctx.has_sent_initial_response
        .store(true, std::sync::atomic::Ordering::SeqCst);

    // Wait for user to submit
    let response = serenity::CollectModalInteraction::new(&ctx.serenity_context.shard)
        .filter(move |d| d.data.custom_id == interaction_id)
        .timeout(timeout.unwrap_or(std::time::Duration::from_secs(3600)))
        .await
        .unwrap();

    // Send acknowledgement so that the pop-up is closed
    response
        .create_interaction_response(ctx.serenity_context, |b| {
            b.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;

    Ok(Some(
        M::parse(response.data.clone()).map_err(serenity::Error::Other)?,
    ))
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
    ///
    /// Optionally takes an initialized instance as pre-filled values of this modal (see
    /// [`Self::execute_with_defaults()`] for more info)
    fn create(
        defaults: Option<Self>,
        custom_id: String,
    ) -> serenity::CreateInteractionResponse<'static>;

    /// Parses a received modal submit interaction into this type
    ///
    /// Returns an error if a field was missing. This should never happen, because Discord will only
    /// let users submit when all required fields are filled properly
    fn parse(data: serenity::ModalSubmitInteractionData) -> Result<Self, &'static str>;

    /// Calls `execute_modal(ctx, None, None)`. See [`execute_modal`]
    // TODO: add execute_with_defaults? Or add a `defaults: Option<Self>` param?
    async fn execute<U: Send + Sync, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
    ) -> Result<Option<Self>, serenity::Error> {
        execute_modal(ctx, None::<Self>, None).await
    }

    /// Calls `execute_modal(ctx, Some(defaults), None)`. See [`execute_modal`]
    // TODO: deprecate this in favor of execute_modal()?
    async fn execute_with_defaults<U: Send + Sync, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
        defaults: Self,
    ) -> Result<Option<Self>, serenity::Error> {
        execute_modal(ctx, Some(defaults), None).await
    }
}
