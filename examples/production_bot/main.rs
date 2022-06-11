#[tokio::main]
async fn main() {
    let mut commands = vec![command1(), command2()];
    
    for &locale in &["your", "locales", "here"] {
        gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, locale);
        for command in &mut commands {
            command.name_localized.insert(locale, gettextrs::gettext(command.name));
            if let Some(description) = command.inline_help {
                command.description_localized.insert(locale, gettextrs::gettext(description));
            }
            for parameter in &mut command.parameters {
                parameter.name_localized.insert(locale, gettextrs::gettext(parameter.name));
                if let Some(description) = parameter.description {
                    parameter.description_localized.insert(locale, gettextrs::gettext(description));
                }
            }
        }
    }

    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions { commands, ..Default::default() })
        // ...
}
