use crate::{Context, Data, Error};

type FluentBundle = fluent::bundle::FluentBundle<
    fluent::FluentResource,
    intl_memoizer::concurrent::IntlLangMemoizer,
>;

pub struct Translations {
    main: FluentBundle,
    other: std::collections::HashMap<String, FluentBundle>,
}

fn format(bundle: &FluentBundle, pattern: &fluent_syntax::ast::Pattern<&str>) -> String {
    bundle
        .format_pattern(pattern, None, &mut vec![])
        .into_owned()
}

pub fn get(ctx: Context<'_>, key: &str) -> String {
    let translations = &ctx.data().translations;
    let bundle = ctx
        .locale()
        .and_then(|locale| translations.other.get(locale))
        .unwrap_or(&translations.main);

    match bundle.get_message(key).and_then(|m| m.value()) {
        Some(pattern) => format(bundle, pattern),
        None => {
            log::warn!("unknown fluent key `{}`", key);
            key.to_string()
        }
    }
}

pub fn read_ftl() -> Result<Translations, Error> {
    fn read_single_ftl(path: &std::path::Path) -> Result<(String, FluentBundle), Error> {
        // Extract locale from filename
        let locale = path.file_stem().ok_or("invalid .ftl filename")?;
        let locale = locale.to_str().ok_or("invalid filename UTF-8")?;

        // Load .ftl resource
        let file_contents = std::fs::read_to_string(&path)?;
        let resource = fluent::FluentResource::try_new(file_contents)
            .map_err(|(_, e)| format!("failed to parse {:?}: {:?}", path, e))?;

        // Associate .ftl resource with locale and bundle it
        let mut bundle = FluentBundle::new_concurrent(vec![locale
            .parse()
            .map_err(|e| format!("invalid locale `{}`: {}", locale, e))?]);
        bundle
            .add_resource(resource)
            .map_err(|e| format!("failed to add resource to bundle: {:?}", e))?;

        Ok((locale.to_string(), bundle))
    }

    Ok(Translations {
        main: read_single_ftl("translations/en-US.ftl".as_ref())?.1,
        other: std::fs::read_dir("translations")?
            .map(|file| read_single_ftl(&file?.path()))
            .collect::<Result<_, _>>()?,
    })
}

pub fn apply_translations(
    translations: &Translations,
    commands: &mut [poise::Command<Data, Error>],
) {
    for command in &mut *commands {
        // Add localizations
        for (locale, bundle) in &translations.other {
            let msg = match bundle.get_message(&command.name) {
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
                    format(bundle, msg.get_attribute(&parameter.name).unwrap().value()),
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
                            bundle.get_message(&choice.name).unwrap().value().unwrap(),
                        ),
                    );
                }
            }
        }

        // Override primary names and descriptions to main translation file
        let bundle = &translations.main;
        let msg = match bundle.get_message(&command.name) {
            Some(x) => x,
            None => continue, // no localization entry => skip localization
        };
        command.name = format(bundle, msg.value().unwrap());
        command.description = Some(format(
            bundle,
            msg.get_attribute("description").unwrap().value(),
        ));
        for parameter in &mut command.parameters {
            parameter.name = format(bundle, msg.get_attribute(&parameter.name).unwrap().value());
            parameter.description = Some(format(
                bundle,
                msg.get_attribute(&format!("{}-description", parameter.name))
                    .unwrap()
                    .value(),
            ));
            for choice in &mut parameter.choices {
                choice.name = format(
                    bundle,
                    bundle.get_message(&choice.name).unwrap().value().unwrap(),
                );
            }
        }
    }
}
