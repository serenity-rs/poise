use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use poise::ChoiceParameter;

#[derive(ChoiceParameter)]
pub enum WelcomeChoice {
    #[name = "Welcome to our cool server! Ask me if you need help"]
    #[name_localized(
        "de",
        "Willkommen auf unserem coolen Server! Frag mich, falls du Hilfe brauchst"
    )]
    A,
    #[name = "Welcome to the club, you're now a good person. Well, I hope."]
    #[name_localized(
        "de",
        "Willkommen im Club, du bist jetzt ein guter Mensch. Naja, hoffentlich."
    )]
    B,
    #[name = "I hope that you brought a controller to play together!"]
    #[name_localized("de", "Ich hoffe du hast einen Controller zum Spielen mitgebracht!")]
    C,
    #[name = "Hey, do you want a coffee?"]
    #[name_localized("de", "Hey, willst du einen Kaffee?")]
    D,
}

/// Welcome a user
#[poise::command(
    slash_command,
    name_localized("de", "begrüßen"),
    description_localized("de", "Einen Nutzer begrüßen")
)]
pub async fn welcome(
    ctx: Context<'_>,
    #[name_localized("de", "nutzer")]
    #[description_localized("de", "Der zu begrüßende Nutzer")]
    #[description = "The user to welcome"]
    user: serenity::User,
    #[name_localized("de", "nachricht")]
    #[description_localized("de", "Die versendete Nachricht")]
    #[description = "The message to send"]
    message: WelcomeChoice,
) -> Result<(), Error> {
    let message = message
        .localized_name(ctx.locale().unwrap_or(""))
        .unwrap_or_else(|| message.name());
    ctx.say(format!("<@{}> {}", user.id.0, message)).await?;
    Ok(())
}
