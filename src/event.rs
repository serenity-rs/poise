//! Provides a utility EventHandler that generates [`Event`] enum instances for incoming events.

use crate::{serenity_prelude as serenity, BoxFuture};

/// A [`serenity::EventHandler`] implementation that wraps every received event into the [`Event`]
/// enum and propagates it to a callback.
///
/// Packaging every event into a singular type can make it easier to pass around and process.
pub struct EventWrapper<F>(pub F)
where
    // gotta have this generic bound in the struct as well, or type inference will break down the line
    F: Send + Sync + for<'a> Fn(serenity::Context, Event<'a>) -> BoxFuture<'a, ()>;

/// Small macro to concisely generate the EventWrapper code while handling every possible event
macro_rules! event {
    ($lt1:lifetime $(
        $( #[$attr:meta] )?
        $fn_name:ident $(<$lt2:lifetime>)? => $variant_name:ident { $( $arg_name:ident: $arg_type:ty ),* },
    )*) => {
        #[serenity::async_trait]
        impl<F> serenity::EventHandler for EventWrapper<F>
        where
            F: Send + Sync + for<'a> Fn(serenity::Context, Event<'a>) -> BoxFuture<'a, ()>
        {
            $(
                $( #[$attr] )?
                async fn $fn_name<'s $(, $lt2)? >(&'s self, ctx: serenity::Context, $( $arg_name: $arg_type, )* ) {
                    (self.0)(ctx, Event::$variant_name { $( $arg_name, )* }).await
                }
            )*
        }

        /// This enum stores every possible event that a [`serenity::EventHandler`] can receive.
        ///
        /// Passed to the stored callback by [`EventWrapper`].
        #[allow(clippy::large_enum_variant)]
        #[allow(missing_docs)]
        #[derive(Debug, Clone)]
        pub enum Event<$lt1> {
            $(
                $( #[$attr] )?
                $variant_name { $( $arg_name: $arg_type ),* },
            )*
            // #[non_exhaustive] forbids struct update syntax for ?? reason
            #[doc(hidden)]
            __NonExhaustive,
        }

        impl Event<'_> {
            /// Return the name of the event type
            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        $( #[$attr] )?
                        Self::$variant_name { .. } => stringify!($variant_name),
                    )*
                    Self::__NonExhaustive => panic!(),
                }
            }

            /// Runs this event in the given [`serenity::EventHandler`]
            pub async fn dispatch(self, ctx: serenity::Context, handler: &dyn serenity::EventHandler) {
                match self {
                    $(
                        $( #[$attr] )?
                        Self::$variant_name { $( $arg_name ),* } => {
                            handler.$fn_name( ctx, $( $arg_name ),* ).await;
                        }
                    )*
                    Self::__NonExhaustive => panic!(),
                }
            }
        }
    };
}

// generated from serenity/client/event_handler.rs
// with help from vscode multiline editing and some manual cleanup
event! {
    'a
    #[cfg(feature = "cache")]
    cache_ready => CacheReady { guilds: Vec<serenity::GuildId> },
    channel_create<'a> => ChannelCreate { channel: &'a serenity::GuildChannel },
    category_create<'a> => CategoryCreate { category: &'a serenity::ChannelCategory },
    category_delete<'a> => CategoryDelete { category: &'a serenity::ChannelCategory },
    channel_delete<'a> => ChannelDelete { channel: &'a serenity::GuildChannel },
    channel_pins_update => ChannelPinsUpdate { pin: serenity::ChannelPinsUpdateEvent },
    #[cfg(feature = "cache")]
    channel_update => ChannelUpdate { old: Option<serenity::Channel>, new: serenity::Channel },
    #[cfg(not(feature = "cache"))]
    channel_update => ChannelUpdate { new: serenity::Channel },
    guild_ban_addition => GuildBanAddition { guild_id: serenity::GuildId, banned_user: serenity::User },
    guild_ban_removal => GuildBanRemoval { guild_id: serenity::GuildId, unbanned_user: serenity::User },
    #[cfg(feature = "cache")]
    guild_create => GuildCreate { guild: serenity::Guild, is_new: bool },
    #[cfg(not(feature = "cache"))]
    guild_create => GuildCreate { guild: serenity::Guild },
    #[cfg(feature = "cache")]
    guild_delete => GuildDelete { incomplete: serenity::UnavailableGuild, full: Option<serenity::Guild> },
    #[cfg(not(feature = "cache"))]
    guild_delete => GuildDelete { incomplete: serenity::UnavailableGuild },
    guild_emojis_update => GuildEmojisUpdate { guild_id: serenity::GuildId, current_state: std::collections::HashMap<serenity::EmojiId, serenity::Emoji> },
    guild_integrations_update => GuildIntegrationsUpdate { guild_id: serenity::GuildId },
    guild_member_addition => GuildMemberAddition { new_member: serenity::Member },
    #[cfg(feature = "cache")]
    guild_member_removal => GuildMemberRemoval { guild_id: serenity::GuildId, user: serenity::User, member_data_if_available: Option<serenity::Member> },
    #[cfg(not(feature = "cache"))]
    guild_member_removal => GuildMemberRemoval { guild_id: serenity::GuildId, user: serenity::User },
    #[cfg(feature = "cache")]
    guild_member_update => GuildMemberUpdate { old_if_available: Option<serenity::Member>, new: serenity::Member },
    #[cfg(not(feature = "cache"))]
    guild_member_update => GuildMemberUpdate { data: serenity::GuildMemberUpdateEvent },
    guild_members_chunk => GuildMembersChunk { chunk: serenity::GuildMembersChunkEvent },
    guild_role_create => GuildRoleCreate { new: serenity::Role },
    #[cfg(feature = "cache")]
    guild_role_delete => GuildRoleDelete { guild_id: serenity::GuildId, removed_role_id: serenity::RoleId, removed_role_data_if_available: Option<serenity::Role> },
    #[cfg(not(feature = "cache"))]
    guild_role_delete => GuildRoleDelete { guild_id: serenity::GuildId, removed_role_id: serenity::RoleId },
    #[cfg(feature = "cache")]
    guild_role_update => GuildRoleUpdate { old_data_if_available: Option<serenity::Role>, new: serenity::Role },
    #[cfg(not(feature = "cache"))]
    guild_role_update => GuildRoleUpdate { new: serenity::Role },
    guild_stickers_update => GuildStickersUpdate { guild_id: serenity::GuildId, current_state: std::collections::HashMap<serenity::StickerId, serenity::Sticker>},
    guild_unavailable => GuildUnavailable { guild_id: serenity::GuildId },
    #[cfg(feature = "cache")]
    guild_update => GuildUpdate { old_data_if_available: Option<serenity::Guild>, new_but_incomplete: serenity::PartialGuild },
    #[cfg(not(feature = "cache"))]
    guild_update => GuildUpdate { new_but_incomplete: serenity::PartialGuild },
    integration_create => IntegrationCreate { integration: serenity::Integration },
    integration_update => IntegrationUpdate { integration: serenity::Integration },
    integration_delete => IntegrationDelete { integration_id: serenity::IntegrationId, guild_id: serenity::GuildId, application_id: Option<serenity::ApplicationId> },
    invite_create => InviteCreate { data: serenity::InviteCreateEvent },
    invite_delete => InviteDelete { data: serenity::InviteDeleteEvent },
    message => Message { new_message: serenity::Message },
    message_delete => MessageDelete { channel_id: serenity::ChannelId, deleted_message_id: serenity::MessageId, guild_id: Option<serenity::GuildId> },
    message_delete_bulk => MessageDeleteBulk { channel_id: serenity::ChannelId, multiple_deleted_messages_ids: Vec<serenity::MessageId>, guild_id: Option<serenity::GuildId> },
    #[cfg(feature = "cache")]
    message_update => MessageUpdate { old_if_available: Option<serenity::Message>, new: Option<serenity::Message>, event: serenity::MessageUpdateEvent },
    #[cfg(not(feature = "cache"))]
    message_update => MessageUpdate { event: serenity::MessageUpdateEvent },
    reaction_add => ReactionAdd { add_reaction: serenity::Reaction },
    reaction_remove => ReactionRemove { removed_reaction: serenity::Reaction },
    reaction_remove_all => ReactionRemoveAll { channel_id: serenity::ChannelId, removed_from_message_id: serenity::MessageId },
    presence_replace => PresenceReplace { new_presences: Vec<serenity::Presence> },
    presence_update => PresenceUpdate { new_data: serenity::Presence },
    ready => Ready { data_about_bot: serenity::Ready },
    resume => Resume { event: serenity::ResumedEvent },
    shard_stage_update => ShardStageUpdate { update: serenity::ShardStageUpdateEvent },
    stage_instance_create => StageInstanceCreate { stage_instance: serenity::StageInstance },
    stage_instance_delete => StageInstanceDelete { stage_instance: serenity::StageInstance },
    stage_instance_update => StageInstanceUpdate { stage_instance: serenity::StageInstance },
    thread_create => ThreadCreate { thread: serenity::GuildChannel },
    thread_delete => ThreadDelete { thread: serenity::PartialGuildChannel },
    thread_list_sync => ThreadListSync { thread_list_sync: serenity::ThreadListSyncEvent },
    thread_member_update => ThreadMemberUpdate { thread_member: serenity::ThreadMember },
    thread_members_update => ThreadMembersUpdate { thread_members_update: serenity::ThreadMembersUpdateEvent },
    thread_update => ThreadUpdate { thread: serenity::GuildChannel },
    typing_start => TypingStart { event: serenity::TypingStartEvent },
    unknown => Unknown { name: String, raw: serenity::json::Value },
    #[cfg(feature = "cache")]
    user_update => UserUpdate { old_data: serenity::CurrentUser, new: serenity::CurrentUser },
    #[cfg(not(feature = "cache"))]
    user_update => UserUpdate { new: serenity::CurrentUser },
    voice_server_update => VoiceServerUpdate { update: serenity::VoiceServerUpdateEvent },
    #[cfg(feature = "cache")]
    voice_state_update => VoiceStateUpdate { old: Option<serenity::VoiceState>, new: serenity::VoiceState },
    #[cfg(not(feature = "cache"))]
    voice_state_update => VoiceStateUpdate { new: serenity::VoiceState },
    webhook_update => WebhookUpdate { guild_id: serenity::GuildId, belongs_to_channel_id: serenity::ChannelId },
    interaction_create => InteractionCreate { interaction: serenity::Interaction },
}
