//! Modal trait and utility items for implementing it (mainly for the derive macro)

use crate::serenity_prelude as serenity;

/// The resolved data for selected options in a [`MentionableSelect`][ms] component.
///
/// [`User`][user] objects resolved from modal interactions include [`PartialMember`][pm]
/// data in the `member` field.
///
/// [ms]: crate::serenity_prelude::ComponentType::MentionableSelect
/// [user]: crate::serenity_prelude::User
/// [pm]: crate::serenity_prelude::PartialMember
#[derive(Clone, Debug, Default)]
pub struct Mentionables {
    /// The resolved users.
    pub users: Vec<serenity::User>,
    /// The resolved roles.
    pub roles: Vec<serenity::Role>,
}

/// Meant for use in derived [`Modal::parse`] implementation.
///
/// Used to return resolved modal interaction data for a component after parsing.
#[doc(hidden)]
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct ModalDataResolved {
    /// The user input from a text input component.
    pub text: Option<String>,
    /// The resolved `Attachment`s from a file upload component.
    pub attachments: Option<Vec<serenity::Attachment>>,
    /// The selected `String`s from a string select menu component.
    pub strings: Option<Vec<String>>,
    /// The resolved `User`s from a user select menu component.
    pub users: Option<Vec<serenity::User>>,
    /// The resolved `Role`s from a role select menu component.
    pub roles: Option<Vec<serenity::Role>>,
    /// The resolved `User`s and `Role`s from a mentionable select menu component.
    pub mentionables: Option<Mentionables>,
    /// The `GenericChannelId` values from a channel select menu component.
    /// Resolved data is not used here because `GenericInteractionChannel` includes
    /// non-exhaustive structs, which would make it impossible to define defaults.
    pub channels: Option<Vec<serenity::GenericChannelId>>,
    /// The `String` value of the option selected from a radio group component.
    pub radio_option: Option<String>,
    /// The `String` values of the options selected from a checkbox group component.
    pub checkbox_options: Option<Vec<String>>,
    /// The `bool` value representing the state of a checkbox component:
    /// `true` if checked, `false` if unchecked.
    pub checked: bool,
}

impl ModalDataResolved {
    /// Used by [`find_modal_data`] to retrieve resolved attachment data from a
    /// `FileUpload` component via _take_.
    #[doc(hidden)]
    fn extract_attachments(
        file_upload: &serenity::all::FileUpload,
        resolved: &mut serenity::CommandDataResolved,
    ) -> Self {
        let mut attachments = Vec::new();
        for value in &file_upload.values {
            if let Some(attachment) = resolved.attachments.remove(value) {
                attachments.push(attachment);
            }
        }
        let attachments = if attachments.is_empty() {
            None
        } else {
            Some(attachments)
        };
        Self {
            attachments,
            ..Default::default()
        }
    }

    /// Used by [`find_modal_data`] to retrieve `values` from [`StringSelect`][ss] components
    /// and resolved data from all other [`SelectMenu`][sm] components.
    ///
    /// `User` and `Role` entity data is _cloned_ since resolved data will be shared between
    /// components when the same entity is selected in multiple components.
    ///
    /// Logs a warning if a value from a component cannot be parsed and used to retrieve the
    /// resolved data for that ID.
    ///
    /// [sm]: crate::serenity_prelude::all::SelectMenu
    /// [ss]: crate::serenity_prelude::ComponentType::StringSelect
    #[doc(hidden)]
    fn extract_selections(
        select_menu: &mut serenity::all::SelectMenu,
        resolved: &serenity::CommandDataResolved,
    ) -> Self {
        match select_menu.kind {
            serenity::ComponentType::StringSelect => {
                let strings = match std::mem::take(&mut select_menu.values) {
                    val if val.is_empty() => None,
                    val => Some(val.into_vec()),
                };
                Self {
                    strings,
                    ..Default::default()
                }
            }
            serenity::ComponentType::UserSelect => {
                let mut users = Vec::new();
                for value in &select_menu.values {
                    if let Ok(id) = value.parse::<u64>() {
                        if let Some(user) = resolved.users.get(&id.into()) {
                            let mut user = user.clone();
                            if let Some(partial_member) = resolved.members.get(&id.into()) {
                                user.member = Some(Box::new(partial_member.clone()));
                            }
                            users.push(user);
                        }
                    } else {
                        tracing::warn!(
                            "Failed to parse `{value}` into u64 and retrieve resolved data"
                        )
                    }
                }
                let users = if users.is_empty() { None } else { Some(users) };
                Self {
                    users,
                    ..Default::default()
                }
            }
            serenity::ComponentType::RoleSelect => {
                let mut roles = Vec::new();
                for value in &select_menu.values {
                    if let Ok(id) = value.parse::<u64>() {
                        if let Some(role) = resolved.roles.get(&id.into()) {
                            roles.push(role.clone());
                        }
                    } else {
                        tracing::warn!(
                            "Failed to parse `{value}` into u64 and retrieve resolved data"
                        )
                    }
                }
                let roles = if roles.is_empty() { None } else { Some(roles) };
                Self {
                    roles,
                    ..Default::default()
                }
            }
            serenity::ComponentType::MentionableSelect => {
                let mut users = Vec::new();
                let mut roles = Vec::new();
                for value in &select_menu.values {
                    if let Ok(id) = value.parse::<u64>() {
                        if let Some(user) = resolved.users.get(&id.into()) {
                            let mut user = user.clone();
                            if let Some(partial_member) = resolved.members.get(&id.into()) {
                                user.member = Some(Box::new(partial_member.clone()));
                            }
                            users.push(user);
                        } else if let Some(role) = resolved.roles.get(&id.into()) {
                            roles.push(role.clone());
                        }
                    } else {
                        tracing::warn!(
                            "Failed to parse `{value}` into u64 and retrieve resolved data"
                        )
                    }
                }
                let mentionables = if users.is_empty() && roles.is_empty() {
                    None
                } else {
                    Some(Mentionables { users, roles })
                };
                Self {
                    mentionables,
                    ..Default::default()
                }
            }
            serenity::ComponentType::ChannelSelect => {
                let channels = if resolved.channels.is_empty() {
                    None
                } else {
                    let mut channels = Vec::new();
                    for channel in &resolved.channels {
                        channels.push(channel.id());
                    }
                    Some(channels)
                };
                Self {
                    channels,
                    ..Default::default()
                }
            }
            _ => Self::default(),
        }
    }
}

impl From<&mut serenity::all::InputText> for ModalDataResolved {
    fn from(value: &mut serenity::all::InputText) -> Self {
        let text = match std::mem::take(&mut value.value) {
            Some(val) if val.is_empty() => None,
            Some(val) => Some(val.into_string()),
            None => None,
        };
        Self {
            text,
            ..Default::default()
        }
    }
}

impl From<&mut serenity::all::RadioGroup> for ModalDataResolved {
    fn from(value: &mut serenity::all::RadioGroup) -> Self {
        let radio_option = match std::mem::take(&mut value.value) {
            Some(val) if val.is_empty() => None,
            Some(val) => Some(val.into_string()),
            None => None,
        };
        Self {
            radio_option,
            ..Default::default()
        }
    }
}

impl From<&mut serenity::all::CheckboxGroup> for ModalDataResolved {
    fn from(value: &mut serenity::all::CheckboxGroup) -> Self {
        let checkbox_options = match std::mem::take(&mut value.values) {
            val if val.is_empty() => None,
            val => Some(val.into_vec()),
        };
        Self {
            checkbox_options,
            ..Default::default()
        }
    }
}

impl From<&mut serenity::all::Checkbox> for ModalDataResolved {
    fn from(value: &mut serenity::all::Checkbox) -> Self {
        Self {
            checked: value.value,
            ..Default::default()
        }
    }
}

/// Meant for use in derived [`Modal::parse`] implementation.
///
/// Retrieves the resolved modal interaction data from the component that has the given
/// `custom_id`, or logs a warning if the component cannot be found.
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
                        return ModalDataResolved::from(input_text);
                    }
                }
                serenity::LabelComponent::FileUpload(file_upload) => {
                    if file_upload.custom_id == custom_id {
                        return ModalDataResolved::extract_attachments(
                            file_upload,
                            &mut data.resolved,
                        );
                    }
                }
                serenity::LabelComponent::SelectMenu(select_menu) => {
                    if select_menu.custom_id == custom_id {
                        match select_menu.kind {
                            serenity::ComponentType::StringSelect
                            | serenity::ComponentType::UserSelect
                            | serenity::ComponentType::RoleSelect
                            | serenity::ComponentType::ChannelSelect
                            | serenity::ComponentType::MentionableSelect => {
                                return ModalDataResolved::extract_selections(
                                    select_menu,
                                    &data.resolved,
                                );
                            }
                            _ => continue,
                        }
                    }
                }
                serenity::LabelComponent::RadioGroup(radio_group) => {
                    if radio_group.custom_id == custom_id {
                        return ModalDataResolved::from(radio_group);
                    }
                }
                serenity::LabelComponent::CheckboxGroup(checkbox_group) => {
                    if checkbox_group.custom_id == custom_id {
                        return ModalDataResolved::from(checkbox_group);
                    }
                }
                serenity::LabelComponent::Checkbox(checkbox) => {
                    if checkbox.custom_id == custom_id {
                        return ModalDataResolved::from(checkbox);
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
/// Note: A modal must be the first response to a command. You cannot send any messages before,
/// or the modal will fail.
///
/// This function:
/// 1. Sends the modal via [`Modal::create()`]
/// 2. Waits for the user to submit via [`serenity::ModalInteractionCollector`]
/// 3. Acknowledges the submitted data so that Discord closes the pop-up for the user
/// 4. Parses the submitted data via [`Modal::parse()`]
///
/// If you need more specialized behavior, you can copy the implementation of this function
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
/// 1. Sends the modal via [`Modal::create()`] as a mci interaction response
/// 2. Waits for the user to submit via [`serenity::ModalInteractionCollector`]
/// 3. Acknowledges the submitted data so that Discord closes the pop-up for the user
/// 4. Parses the submitted data via [`Modal::parse()`]
///
/// If you need more specialized behavior, you can copy the implementation of this function
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
///     #[paragraph] // Switches from single-line to multi-line text box
///     second_input: Option<String>, // Option means optional input
///     #[name = "File upload"]
///     #[file_upload] // Allows user to upload up to 10 files
///     #[min_values = 2] // Min number of files (0-10 for files)
///     #[max_values = 5]
///     third_input: Vec<serenity::Attachment>,
///     #[name = "String select menu"]
///     #[string_select("Option 1", "Option 2")] // Selectable strings
///     #[min_values = 2] // Min number of selections required (0-25 for select menus)
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
    /// Returns an interaction response builder which creates the modal for this type.
    ///
    /// Optionally takes an initialized instance as pre-filled values of this modal. See
    /// [`Self::execute_with_defaults()`] for more info.
    fn create(
        defaults: Option<Self>,
        custom_id: String,
    ) -> serenity::CreateInteractionResponse<'static>;

    /// Parses a received modal submit interaction into this type.
    ///
    /// Returns an error if a field was missing. This should never happen, because Discord will only
    /// let users submit when all required fields are filled properly.
    fn parse(data: serenity::ModalInteractionData) -> Self;

    /// Calls `execute_modal(ctx, None, None)`. See [`execute_modal()`].
    ///
    /// For a variant that is triggered on component interactions, see [`execute_modal_on_component_interaction`].
    // TODO: add execute_with_defaults? Or add a `defaults: Option<Self>` param?
    async fn execute<U: Send + Sync + 'static, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
    ) -> Result<Option<Self>, serenity::Error> {
        execute_modal(ctx, None::<Self>, None).await
    }

    /// Calls `execute_modal(ctx, Some(defaults), None)`. See [`execute_modal()`] and
    /// [`Modal`][crate::macros::Modal#specifying-defaults].
    // TODO: deprecate this in favor of execute_modal()?
    async fn execute_with_defaults<U: Send + Sync + 'static, E>(
        ctx: crate::ApplicationContext<'_, U, E>,
        defaults: Self,
    ) -> Result<Option<Self>, serenity::Error> {
        execute_modal(ctx, Some(defaults), None).await
    }
}
