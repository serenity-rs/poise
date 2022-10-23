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

/// See [`Modal::execute`]
async fn execute<U: Send + Sync, E, M: Modal>(
    ctx: crate::ApplicationContext<'_, U, E>,
    defaults: Option<M>,
) -> Result<M, serenity::Error> {
    let interaction = ctx.interaction.unwrap();

    // Send modal
    interaction
        .create_interaction_response(ctx.discord, M::create(defaults))
        .await?;
    ctx.has_sent_initial_response
        .store(true, std::sync::atomic::Ordering::SeqCst);

    // Wait for user to submit
    let response = serenity::CollectModalInteraction::new(&ctx.discord.shard)
        .author_id(interaction.user.id)
        .collect_single()
        .await
        .unwrap();

    // Send acknowledgement so that the pop-up is closed
    response
        .create_interaction_response(
            ctx.discord,
            serenity::CreateInteractionResponse::Acknowledge,
        )
        .await?;

    M::parse(response.data.clone()).map_err(serenity::Error::Other)
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
    fn create(defaults: Option<Self>) -> serenity::CreateInteractionResponse;

    /// Parses a received modal submit interaction into this type
    ///
    /// Returns an error if a field was missing. This should never happen, because Discord will only
    /// let users submit when all required fields are filled properly
    fn parse(data: serenity::ModalSubmitInteractionData) -> Result<Self, &'static str>;

    /// Convenience function for showing the modal and waiting for a response
    ///
    /// Note: a modal must be the first response to a command. You cannot send any messages before,
    /// or the modal will fail
    ///
    /// This function:
    /// 1. sends the modal via [`Self::create()`]
    /// 2. waits for the user to submit via [`serenity::CollectModalInteraction`]
    /// 3. acknowledges the submitted data so that Discord closes the pop-up for the user
    /// 4. parses the submitted data via [`Self::parse()`], wrapping errors in [`serenity::Error::Other`]
    // TODO: add execute_with_defaults? Or add a `defaults: Option<Self>` param?
    async fn execute<U: Send + Sync, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
    ) -> Result<Self, serenity::Error> {
        execute(ctx, None::<Self>).await
    }

    /// Like [`Self::execute()`], but with a parameter to set default values for the fields.
    ///
    /// ```rust
    /// # async fn _foo(ctx: poise::ApplicationContext<'_, (), ()>) -> Result<(), serenity::Error> {
    /// # use poise::Modal as _;
    /// #[derive(Default, poise::Modal)]
    /// struct MyModal {
    ///     field_1: String,
    ///     field_2: String,
    /// }
    ///
    /// # let ctx: poise::ApplicationContext<'static, (), ()> = todo!();
    /// MyModal::execute_with_defaults(ctx, MyModal {
    ///     field_1: "Default value".into(),
    ///     ..Default::default()
    /// }).await?;
    /// # Ok(()) }
    /// ```
    async fn execute_with_defaults<U: Send + Sync, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
        defaults: Self,
    ) -> Result<Self, serenity::Error> {
        execute(ctx, Some(defaults)).await
    }
}
