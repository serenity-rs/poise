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
    #[name_localized(
        "es-ES",
        "¡Bienvenido a nuestro genial servidor! Pregúntame si necesitas ayuda"
    )]
    A,
    #[name = "Welcome to the club, you're now a good person. Well, I hope."]
    #[name_localized(
        "de",
        "Willkommen im Club, du bist jetzt ein guter Mensch. Naja, hoffentlich."
    )]
    #[name_localized(
        "es-ES",
        "Bienvenido al club, ahora eres una buena persona. Bueno, eso espero."
    )]
    B,
    #[name = "I hope that you brought a controller to play together!"]
    #[name_localized("de", "Ich hoffe du hast einen Controller zum Spielen mitgebracht!")]
    #[name_localized("es-ES", "Espero que hayas traído un mando para jugar juntos.")]
    C,
    #[name = "Hey, do you want a coffee?"]
    #[name_localized("de", "Hey, willst du einen Kaffee?")]
    #[name_localized("es-ES", "Oye, ¿Quieres un café?")]
    D,
}

/// Welcome a user
#[poise::command(
    slash_command,
    name_localized("de", "begrüßen"),
    name_localized("es-ES", "saludar"),
    description_localized("de", "Einen Nutzer begrüßen"),
    description_localized("es-ES", "Saludar a un usuario")
)]
pub async fn welcome(
    ctx: Context<'_>,
    #[name_localized("de", "nutzer")]
    #[description_localized("de", "Der zu begrüßende Nutzer")]
    #[name_localized("es-ES", "usuario")]
    #[description_localized("es-ES", "El usuario a saludar")]
    #[description = "The user to welcome"]
    user: serenity::User,
    #[name_localized("de", "nachricht")]
    #[description_localized("de", "Die versendete Nachricht")]
    #[name_localized("es-ES", "mensaje")]
    #[description_localized("es-ES", "El mensaje enviado")]
    #[description = "The message to send"]
    message: WelcomeChoice,
) -> Result<(), Error> {
    let message = message
        .localized_name(ctx.locale().unwrap_or(""))
        .unwrap_or_else(|| message.name());
    ctx.say(format!("<@{}> {}", user.id.0, message)).await?;
    Ok(())
}
