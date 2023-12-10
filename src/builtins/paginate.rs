//! Sample pagination implementation

use crate::serenity_prelude as serenity;

/// This is an example implementation of pagination. To tweak the behavior, copy the source code and
/// adjust to your needs:
/// - change embed appearance
/// - use different emojis for the navigation buttons
/// - add more navigation buttons
/// - change timeout duration
/// - add a page selector dropdown
/// - use reactions instead of buttons
/// - remove message after navigation timeout
/// - ...
///
/// Note: this is a long-running function. It will only return once the timeout for navigation
/// button interactions has been reached.
///
/// # Example
///
/// ```rust,no_run
/// # async fn _test(ctx: poise::Context<'_, (), serenity::Error>) -> Result<(), serenity::Error> {
/// let pages = &[
///     "Content of first page",
///     "Content of second page",
///     "Content of third page",
///     "Content of fourth page",
/// ];
///
/// poise::samples::paginate(ctx, pages).await?;
/// # Ok(()) }
/// ```
///
/// ![Screenshot of output](https://i.imgur.com/JGFDveA.png)
pub async fn paginate<U, E>(
    ctx: crate::Context<'_, U, E>,
    pages: &[&str],
) -> Result<(), serenity::Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // Send the embed with the first page as content
    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&prev_button_id).emoji('◀'),
            serenity::CreateButton::new(&next_button_id).emoji('▶'),
        ]);

        crate::CreateReply::default()
            .embed(serenity::CreateEmbed::default().description(pages[0]))
            .components(vec![components])
    };

    ctx.send(reply).await?;

    // Loop through incoming interactions with the navigation buttons
    let mut current_page = 0;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id.as_str() == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id.as_str() == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(serenity::CreateEmbed::new().description(pages[current_page])),
                ),
            )
            .await?;
    }

    Ok(())
}
