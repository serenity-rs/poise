//! Modal trait and utility items for implementing it (mainly for the derive macro)

use crate::serenity_prelude as serenity;

/// Meant for use in derived [`Modal::parse`] implementation.
///
/// Used to return resolved modal interaction data for a component after parsing.
#[doc(hidden)]
#[non_exhaustive]
#[derive(Default)]
pub struct ModalDataResolved {
    pub text: Option<String>,
    pub attachments: Option<Vec<serenity::Attachment>>,
    pub strings: Option<Vec<String>>,
    pub users: Option<Vec<serenity::User>>,
    pub roles: Option<Vec<serenity::Role>>,
    pub mentionables: Option<(Vec<serenity::User>, Vec<serenity::Role>)>,
    pub channels: Option<Vec<serenity::GenericInteractionChannel>>,
}

impl ModalDataResolved {
    /// Used by [`find_modal_data`] to retrieve resolved data from a component via _take_.
    #[doc(hidden)]
    fn take_from_modal(
        kind: serenity::ComponentType,
        values: &mut serenity::small_fixed_array::FixedArray<String, u32>,
        resolved: &mut serenity::CommandDataResolved,
    ) -> Self {
        match kind {
            serenity::ComponentType::StringSelect => {
                let strings = match std::mem::take(values) {
                    val if val.is_empty() => None,
                    val => Some(val.into_vec()),
                };
                ModalDataResolved {
                    strings,
                    ..Default::default()
                }
            }
            serenity::ComponentType::ChannelSelect => {
                let channels = match std::mem::take(&mut resolved.channels) {
                    val if val.is_empty() => None,
                    val => Some(val.into_iter().collect::<Vec<_>>()),
                };
                ModalDataResolved {
                    channels,
                    ..Default::default()
                }
            }
            _ => ModalDataResolved::default(),
        }
    }

    /// Used by [`find_modal_data`] to retrieve resolved data from a component via _cloning_.
    #[doc(hidden)]
    fn clone_from_modal(
        kind: serenity::ComponentType,
        values: &serenity::small_fixed_array::FixedArray<String, u32>,
        resolved: &serenity::CommandDataResolved,
    ) -> Self {
        let mut users = Vec::new();
        let mut roles = Vec::new();
        for value in values {
            let id = value.parse::<u64>().unwrap_or_default();
            if let Some(user) = resolved.users.get(&id.into()) {
                users.push(user.clone());
            } else if let Some(role) = resolved.roles.get(&id.into()) {
                roles.push(role.clone());
            }
        }
        let (users, roles, mentionables) =
            if matches!(kind, serenity::ComponentType::MentionableSelect) {
                let mentionables = match (users.len(), roles.len()) {
                    (0, 0) => None,
                    _ => Some((users, roles)),
                };
                (None, None, mentionables)
            } else {
                let users = if users.is_empty() { None } else { Some(users) };
                let roles = if roles.is_empty() { None } else { Some(roles) };
                (users, roles, None)
            };
        ModalDataResolved {
            users,
            roles,
            mentionables,
            ..Default::default()
        }
    }
}

/// Meant for use in derived [`Modal::parse`] implementation.
///
/// Retrieves the resolved modal interaction data from the component that has the given
/// `custom_id`, or logs a warning if the component cannot be found.
///
/// For `InputText`, `FileUpload`, `StringSelect`, and `ChannelSelect` components, _takes_
/// the data out of the component. For `UserSelect`, `RoleSelect`, and `MentionableSelect`
/// components, data is _cloned_ instead since it may be shared between the components.
#[doc(hidden)]
pub fn find_modal_data(
    data: &mut serenity::ModalInteractionData,
    custom_id: &str,
) -> ModalDataResolved {
    for component in data.components.iter_mut() {
        match component {
            serenity::ModalComponent::Label(label) => match &mut label.component {
                serenity::LabelComponent::InputText(input_text) => {
                    if input_text.custom_id == custom_id {
                        let text = if input_text.value.is_empty() {
                            None
                        } else {
                            Some(std::mem::take(&mut input_text.value).into_string())
                        };
                        return ModalDataResolved {
                            text,
                            ..Default::default()
                        };
                    }
                }
                serenity::LabelComponent::FileUpload(file_upload) => {
                    if file_upload.custom_id == custom_id {
                        let attachments = match std::mem::take(&mut data.resolved.attachments) {
                            val if val.is_empty() => None,
                            val => Some(val.into_iter().collect::<Vec<_>>()),
                        };
                        return ModalDataResolved {
                            attachments,
                            ..Default::default()
                        };
                    }
                }
                serenity::LabelComponent::SelectMenu(select_menu) => {
                    if select_menu.custom_id == custom_id {
                        match select_menu.kind {
                            serenity::ComponentType::StringSelect
                            | serenity::ComponentType::ChannelSelect => {
                                return ModalDataResolved::take_from_modal(
                                    select_menu.kind,
                                    &mut select_menu.values,
                                    &mut data.resolved,
                                );
                            }
                            serenity::ComponentType::UserSelect
                            | serenity::ComponentType::RoleSelect
                            | serenity::ComponentType::MentionableSelect => {
                                return ModalDataResolved::clone_from_modal(
                                    select_menu.kind,
                                    &select_menu.values,
                                    &data.resolved,
                                );
                            }
                            _ => continue,
                        }
                    }
                }
                _ => continue,
            },
            _ => continue,
        }
    }
    tracing::warn!("{custom_id} not found in modal response");
    ModalDataResolved::default()
}

/// Underlying code for the modal spawning convenience function which abstracts over the kind of
/// interaction
async fn execute_modal_generic<
    M: Modal,
    F: std::future::Future<Output = Result<(), serenity::Error>>,
>(
    ctx: &serenity::Context,
    create_interaction_response: impl FnOnce(serenity::CreateInteractionResponse<'static>) -> F,
    modal_custom_id: String,
    defaults: Option<M>,
    timeout: Option<std::time::Duration>,
) -> Result<Option<M>, serenity::Error> {
    // Send modal
    create_interaction_response(M::create(defaults, modal_custom_id.clone())).await?;

    // Wait for user to submit
    let response = serenity::collector::ModalInteractionCollector::new(ctx)
        .filter(move |d| d.data.custom_id.as_str() == modal_custom_id)
        .timeout(timeout.unwrap_or(std::time::Duration::from_secs(3600)))
        .await;
    let response = match response {
        Some(x) => x,
        None => return Ok(None),
    };

    // Send acknowledgement so that the pop-up is closed
    response
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge)
        .await?;

    Ok(Some(M::parse(response.data)))
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
/// 2. waits for the user to submit via [`serenity::ModalInteractionCollector`]
/// 3. acknowledges the submitted data so that Discord closes the pop-up for the user
/// 4. parses the submitted data via [`Modal::parse()`]
///
/// If you need more specialized behavior, you can copy paste the implementation of this function
/// and adjust to your needs. The code of this function is just a starting point.
pub async fn execute_modal<U: Send + Sync + 'static, E, M: Modal>(
    ctx: crate::ApplicationContext<'_, U, E>,
    defaults: Option<M>,
    timeout: Option<std::time::Duration>,
) -> Result<Option<M>, serenity::Error> {
    let interaction = ctx.interaction;
    let response = execute_modal_generic(
        ctx.serenity_context(),
        |resp| interaction.create_response(ctx.http(), resp),
        interaction.id.to_string(),
        defaults,
        timeout,
    )
    .await?;
    ctx.has_sent_initial_response
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(response)
}

/// Convenience function for showing the modal on a message interaction and waiting for a response.
///
/// If the user doesn't submit before the timeout expires, `None` is returned.
///
/// This function:
/// 1. sends the modal via [`Modal::create()`] as a mci interaction response
/// 2. waits for the user to submit via [`serenity::ModalInteractionCollector`]
/// 3. acknowledges the submitted data so that Discord closes the pop-up for the user
/// 4. parses the submitted data via [`Modal::parse()`]
///
/// If you need more specialized behavior, you can copy paste the implementation of this function
/// and adjust to your needs. The code of this function is just a starting point.
pub async fn execute_modal_on_component_interaction<M: Modal>(
    ctx: &serenity::Context,
    interaction: serenity::ComponentInteraction,
    defaults: Option<M>,
    timeout: Option<std::time::Duration>,
) -> Result<Option<M>, serenity::Error> {
    execute_modal_generic(
        ctx,
        |resp| interaction.create_response(&ctx.http, resp),
        interaction.id.to_string(),
        defaults,
        timeout,
    )
    .await
}

/// Derivable trait for modal interactions, Discord's version of interactive forms.
///
/// You don't need to implement this trait manually; use `#[derive(poise::Modal)]` instead.
/// See [`Modal`][crate::macros::Modal] for details.
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
/// #[name = "Modal Title"] // Struct name by default
/// #[text = "My *fancy* `modal`, created using [Poise](https://serenity-rs.github.io/) :crab:"]
/// struct MyModal {
///     #[name = "First text input"] // Field name by default
///     #[description = "Displayed under name"] // No description by default
///     #[placeholder = "Your first input goes here"] // No placeholder by default
///     #[min_length = 5] // No length restriction by default (up to 4000 chars)
///     #[max_length = 500]
///     first_input: String,
///     #[name = "Second text input"]
///     #[value = "This one has been pre-filled!"]
///     #[paragraph] // Switches from single-line to multi-line text box
///     second_input: Option<String>, // Option means optional input
///     #[name = "File upload"]
///     #[file_upload] // Allows user to upload up to 10 files
///     #[min_items = 2] // Min number of files (0-10 for files)
///     #[max_items = 5]
///     third_input: Vec<serenity::Attachment>,
///     #[name = "String select menu"]
///     #[string_select("Option 1", "Option 2")] // Selectable strings (defaults to 1)
///     #[min_items = 2] // Min number of selections required (1-25 for select menus)
///     fourth_input: Vec<String>,
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
    fn parse(data: serenity::ModalInteractionData) -> Self;

    /// Calls `execute_modal(ctx, None, None)`. See [`execute_modal`]
    ///
    /// For a variant that is triggered on component interactions, see [`execute_modal_on_component_interaction`].
    // TODO: add execute_with_defaults? Or add a `defaults: Option<Self>` param?
    async fn execute<U: Send + Sync + 'static, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
    ) -> Result<Option<Self>, serenity::Error> {
        execute_modal(ctx, None::<Self>, None).await
    }

    /// Calls `execute_modal(ctx, Some(defaults), None)`. See [`execute_modal`]
    // TODO: deprecate this in favor of execute_modal()?
    async fn execute_with_defaults<U: Send + Sync + 'static, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
        defaults: Self,
    ) -> Result<Option<Self>, serenity::Error> {
        execute_modal(ctx, Some(defaults), None).await
    }
}
