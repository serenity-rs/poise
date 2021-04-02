use crate::BoxFutureBorrowed;
use serde_json::Value;
use serenity::{client::bridge::gateway::event::*, model::prelude::*};
use std::collections::HashMap;

pub struct EventWrapper<F>(pub F)
where
    // gotta have this generic bound in the struct as well, or type inference will break down the line
    F: Send + Sync + for<'a> Fn(serenity::prelude::Context, Event<'a>) -> BoxFutureBorrowed<'a, ()>;

macro_rules! event {
	($lt1:lifetime $(
		$fn_name:ident $(<$lt2:lifetime>)? => $variant_name:ident { $( $arg_name:ident: $arg_type:ty ),* },
	)*) => {
        #[serenity::async_trait]
		impl<F> serenity::prelude::EventHandler for EventWrapper<F>
		where
			F: Send + Sync + for<'a> Fn(serenity::prelude::Context, Event<'a>) -> BoxFutureBorrowed<'a, ()>
		{
			$(
				async fn $fn_name<'s $(, $lt2)? >(&'s self, ctx: serenity::prelude::Context, $( $arg_name: $arg_type, )* ) {
					(self.0)(ctx, Event::$variant_name { $( $arg_name, )* }).await
				}
			)*
		}

		#[allow(clippy::large_enum_variant)]
		#[derive(Debug, Clone)]
		pub enum Event<$lt1> {
			$(
				$variant_name { $( $arg_name: $arg_type ),* },
			)*
		}
	};
}

// generated from https://docs.rs/serenity/0.8.9/src/serenity/client/event_handler.rs.html#12-314
// with help from vscode multiline editing and some manual cleanup
event! {
    'a
    cache_ready => CacheReady { guilds: Vec<GuildId> },
    channel_create<'a> => ChannelCreate { channel: &'a GuildChannel },
    category_create<'a> => CategoryCreate { category: &'a ChannelCategory },
    category_delete<'a> => CategoryDelete { category: &'a ChannelCategory },
    channel_delete<'a> => ChannelDelete { channel: &'a GuildChannel },
    channel_pins_update => ChannelPinsUpdate { pin: ChannelPinsUpdateEvent },
    channel_update => ChannelUpdate { old: Option<Channel>, new: Channel },
    guild_ban_addition => GuildBanAddition { guild_id: GuildId, banned_user: User },
    guild_ban_removal => GuildBanRemoval { guild_id: GuildId, unbanned_user: User },
    guild_create => GuildCreate { guild: Guild, is_new: bool },
    guild_delete => GuildDelete { incomplete: GuildUnavailable, full: Option<Guild> },
    guild_emojis_update => GuildEmojisUpdate { guild_id: GuildId, current_state: HashMap<EmojiId, Emoji> },
    guild_integrations_update => GuildIntegrationsUpdate { guild_id: GuildId },
    guild_member_addition => GuildMemberAddition { guild_id: GuildId, new_member: Member },
    guild_member_removal => GuildMemberRemoval { guild_id: GuildId, user: User, member_data_if_available: Option<Member> },
    guild_member_update => GuildMemberUpdate { old_if_available: Option<Member>, new: Member },
    guild_members_chunk => GuildMembersChunk { chunk: GuildMembersChunkEvent },
    guild_role_create => GuildRoleCreate { guild_id: GuildId, new: Role },
    guild_role_delete => GuildRoleDelete { guild_id: GuildId, removed_role_id: RoleId, removed_role_data_if_available: Option<Role> },
    guild_role_update => GuildRoleUpdate { guild_id: GuildId, old_data_if_available: Option<Role>, new: Role },
    guild_unavailable => GuildUnavailable { guild_id: GuildId },
    guild_update => GuildUpdate { old_data_if_available: Option<Guild>, new_but_incomplete: PartialGuild },
    invite_create => InviteCreate { data: InviteCreateEvent },
    invite_delete => InviteDelete { data: InviteDeleteEvent },
    message => Message { new_message: Message },
    message_delete => MessageDelete { channel_id: ChannelId, deleted_message_id: MessageId, guild_id: Option<GuildId> },
    message_delete_bulk => MessageDeleteBulk { channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId>, guild_id: Option<GuildId> },
    message_update => MessageUpdate { old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent },
    reaction_add => ReactionAdd { add_reaction: Reaction },
    reaction_remove => ReactionRemove { removed_reaction: Reaction },
    reaction_remove_all => ReactionRemoveAll { channel_id: ChannelId, removed_from_message_id: MessageId },
    presence_replace => PresenceReplace { new_presences: Vec<Presence> },
    presence_update => PresenceUpdate { new_data: PresenceUpdateEvent },
    ready => Ready { data_about_bot: Ready },
    resume => Resume { event: ResumedEvent },
    shard_stage_update => ShardStageUpdate { update: ShardStageUpdateEvent },
    typing_start => TypingStart { event: TypingStartEvent },
    unknown => Unknown { name: String, raw: Value },
    user_update => UserUpdate { old_data: CurrentUser, new: CurrentUser },
    voice_server_update => VoiceServerUpdate { update: VoiceServerUpdateEvent },
    voice_state_update => VoiceStateUpdate { guild_id: Option<GuildId>, old: Option<VoiceState>, new: VoiceState },
    webhook_update => WebhookUpdate { guild_id: GuildId, belongs_to_channel_id: ChannelId },
}
