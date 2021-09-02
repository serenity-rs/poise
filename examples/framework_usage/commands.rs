use crate::{Context, Error};

/// Vote for something
///
/// Enter `~vote pumpkin` to vote for pumpkins
#[poise::command(slash_command)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "What to vote for"] choice: String,
) -> Result<(), Error> {
    let num_votes = {
        let mut hash_map = ctx.data().votes.lock().unwrap();
        let num_votes = hash_map.entry(choice.clone()).or_default();
        *num_votes += 1;
        *num_votes
    };

    let response = format!(
        "Successfully voted for {0}. {0} now has {1} votes!",
        choice, num_votes
    );
    poise::say_reply(ctx, response).await?;
    Ok(())
}

/// Retrieve number of votes
///
/// Retrieve the number of votes either in general, or for a specific choice:
/// ```
/// ~getvotes
/// ~getvotes pumpkin
/// ```
#[poise::command(slash_command, track_edits, aliases("votes"))]
pub async fn getvotes(
    ctx: Context<'_>,
    #[description = "Choice to retrieve votes for"] choice: Option<String>,
) -> Result<(), Error> {
    if let Some(choice) = choice {
        let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
        let response = match num_votes {
            0 => format!("Nobody has voted for {} yet", choice),
            _ => format!("{} people have voted for {}", num_votes, choice),
        };
        poise::say_reply(ctx, response).await?;
    } else {
        let mut response = String::new();
        for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
            response += &format!("{}: {} votes\n", choice, num_votes);
        }

        if response.is_empty() {
            response += "Nobody has voted for anything yet :(";
        }

        poise::say_reply(ctx, response).await?;
    };

    Ok(())
}

/// Add two numbers
#[poise::command(slash_command, track_edits)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "First operand"] a: f64,
    #[description = "Second operand"] b: f32,
) -> Result<(), Error> {
    poise::say_reply(ctx, format!("Result: {}", a + b as f64)).await?;

    Ok(())
}

#[derive(Debug, poise::SlashChoiceParameter)]
pub enum MyStringChoice {
    #[name = "The first choice"]
    ChoiceA,
    #[name = "The second choice"]
    ChoiceB,
}

/// Dummy command to test slash command choice parameters
#[poise::command(slash_command)]
pub async fn choice(
    ctx: Context<'_>,
    #[description = "The choice you want to choose"] choice: poise::Wrapper<MyStringChoice>,
) -> Result<(), Error> {
    let choice = choice.0;

    poise::say_reply(ctx, format!("You entered {:?}", choice)).await?;
    Ok(())
}

/// Boop the bot!
#[poise::command(slash_command, track_edits)]
pub async fn boop(ctx: Context<'_>) -> Result<(), Error> {
    let uuid_boop = ctx.id();

    poise::send_reply(ctx, |m| {
        m.content("I want some boops!".into()).components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.style(serenity::ButtonStyle::Primary)
                        .label("Boop me!")
                        .custom_id(uuid_boop)
                })
            })
        })
    })
    .await?;

    let mut boop_count = 0;
    while let Some(mci) = serenity::CollectComponentInteraction::new(ctx.discord())
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == uuid_boop.to_string())
        .await
    {
        boop_count += 1;

        let mut msg = mci.message.clone().regular().unwrap();
        msg.edit(ctx.discord(), |m| {
            m.content(format!("Boop count: {}", boop_count))
        })
        .await?;

        mci.create_interaction_response(ctx.discord(), |ir| {
            ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;
    }

    Ok(())
}
