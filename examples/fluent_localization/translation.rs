use crate::{Context, Data, Error, FluentBundle};

fn format(bundle: &FluentBundle, pattern: &fluent_syntax::ast::Pattern<&str>) -> String {
    bundle
        .format_pattern(pattern, None, &mut vec![])
        .into_owned()
}

pub fn get(ctx: Context<'_>, key: &str) -> String {
    let translations = &ctx.data().translations;
    let bundle = translations
        .get(ctx.locale().unwrap_or("en-US"))
        .unwrap_or_else(|| {
            translations
                .get("en-US")
                .expect("no en-US translation found")
        });

    match bundle.get_message(key).and_then(|m| m.value()) {
        Some(pattern) => format(bundle, pattern),
        None => {
            log::warn!("unknown fluent key `{}`", key);
            key.to_string()
        }
    }
}

pub fn read_ftl() -> Result<std::collections::HashMap<String, FluentBundle>, Error> {
    let mut translations = std::collections::HashMap::new();
    for file in std::fs::read_dir("translations")? {
        let file = file?;
        let path = file.path();

        // Extract locale from filename
        let locale = path
            .file_stem()
            .ok_or_else(|| format!("invalid language filename `{:?}`", file.file_name()))?;
        let locale = locale
            .to_str()
            .ok_or_else(|| format!("invalid UTF-8: {:?}", locale))?;

        // Load .ftl resource
        let file_contents = std::fs::read_to_string(&path)?;
        let resource = fluent::FluentResource::try_new(file_contents)
            .map_err(|(_, e)| format!("failed to parse {:?}: {:?}", file.file_name(), e))?;

        // Associate .ftl resource with locale and store it
        let mut bundle = FluentBundle::new_concurrent(vec![locale
            .parse()
            .map_err(|e| format!("invalid locale `{}`: {}", locale, e))?]);
        bundle
            .add_resource(resource)
            .map_err(|e| format!("failed to add resource to bundle: {:?}", e))?;
        translations.insert(locale.to_string(), bundle);
    }
    Ok(translations)
}

pub fn apply_translations(
    translations: &std::collections::HashMap<String, FluentBundle>,
    commands: &mut [poise::Command<Data, Error>],
) {
    for (locale, bundle) in translations {
        for command in &mut *commands {
            let msg = match bundle.get_message(command.name) {
                Some(x) => x,
                None => continue, // no localization entry => skip localization
            };

            command
                .name_localizations
                .insert(locale.clone(), format(bundle, msg.value().unwrap()));
            command.description_localizations.insert(
                locale.clone(),
                format(bundle, msg.get_attribute("description").unwrap().value()),
            );
            for parameter in &mut command.parameters {
                parameter.name_localizations.insert(
                    locale.clone(),
                    format(bundle, msg.get_attribute(parameter.name).unwrap().value()),
                );
                parameter.description_localizations.insert(
                    locale.clone(),
                    format(
                        bundle,
                        msg.get_attribute(&format!("{}-description", parameter.name))
                            .unwrap()
                            .value(),
                    ),
                );
                for choice in &mut parameter.choices {
                    choice.localizations.insert(
                        locale.clone(),
                        format(
                            bundle,
                            bundle.get_message(choice.name).unwrap().value().unwrap(),
                        ),
                    );
                }
            }
        }
    }
}
