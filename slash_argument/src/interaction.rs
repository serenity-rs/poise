use crate::serenity_prelude as serenity;

/// Abstracts over a refernce to an application command interaction or autocomplete interaction
///
/// We need to support autocomplete interactions in
/// `ApplicationContext` because command checks are invoked for autocomplete interactions
/// too: we don't want poise accidentally leaking sensitive information through autocomplete
/// suggestions
// TODO: inline this struct once merged into main branch
#[derive(Copy, Clone, Debug)]
pub enum ApplicationCommandOrAutocompleteInteraction<'a> {
    /// An application command interaction
    ApplicationCommand(&'a serenity::ApplicationCommandInteraction),
    /// An autocomplete interaction
    Autocomplete(&'a serenity::ApplicationCommandInteraction),
}

impl<'a> ApplicationCommandOrAutocompleteInteraction<'a> {
    /// Returns the data field of the underlying interaction
    pub fn data(self) -> &'a serenity::CommandData {
        match self {
            Self::ApplicationCommand(x) => &x.data,
            Self::Autocomplete(x) => &x.data,
        }
    }

    /// Returns the ID of the underlying interaction
    pub fn id(self) -> serenity::InteractionId {
        match self {
            Self::ApplicationCommand(x) => x.id,
            Self::Autocomplete(x) => x.id,
        }
    }

    /// Returns the guild ID of the underlying interaction
    pub fn guild_id(self) -> Option<serenity::GuildId> {
        match self {
            Self::ApplicationCommand(x) => x.guild_id,
            Self::Autocomplete(x) => x.guild_id,
        }
    }

    /// Returns the channel ID of the underlying interaction
    pub fn channel_id(self) -> serenity::ChannelId {
        match self {
            Self::ApplicationCommand(x) => x.channel_id,
            Self::Autocomplete(x) => x.channel_id,
        }
    }

    /// Returns the member field of the underlying interaction
    pub fn member(self) -> Option<&'a serenity::Member> {
        match self {
            Self::ApplicationCommand(x) => x.member.as_ref(),
            Self::Autocomplete(x) => x.member.as_ref(),
        }
    }

    /// Returns the user field of the underlying interaction
    pub fn user(self) -> &'a serenity::User {
        match self {
            Self::ApplicationCommand(x) => &x.user,
            Self::Autocomplete(x) => &x.user,
        }
    }

    /// Returns the inner [`serenity::ApplicationCommandInteraction`] and panics otherwise
    pub fn unwrap(self) -> &'a serenity::ApplicationCommandInteraction {
        match self {
            ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => x,
            ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => {
                panic!("expected application command interaction, got autocomplete interaction")
            }
        }
    }

    /// Returns the locale field of the underlying interaction
    pub fn locale(self) -> &'a str {
        match self {
            ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(x) => &x.locale,
            ApplicationCommandOrAutocompleteInteraction::Autocomplete(x) => &x.locale,
        }
    }
}
