use parking_lot::RwLock;
use serde_json::Value;
use serenity::{client::bridge::gateway::event::*, model::prelude::*};
use std::{collections::HashMap, sync::Arc};

pub struct EventWrapper<F>(pub F);

macro_rules! event {
	($(
		$fn_name:ident => $variant_name:ident { $( $arg_name:ident: $arg_type:ty ),* },
	)*) => {
		impl<F> serenity::prelude::EventHandler for EventWrapper<F>
		where
			F: Send + Sync + Fn(serenity::prelude::Context, Event)
		{
			$(
				fn $fn_name(&self, ctx: serenity::prelude::Context, $( $arg_name: $arg_type, )* ) {
					(self.0)(ctx, Event::$variant_name { $( $arg_name, )* })
				}
			)*
		}

		#[allow(clippy::large_enum_variant)]
		#[derive(Debug, Clone)]
		pub enum Event {
			$(
				$variant_name { $( $arg_name: $arg_type ),* },
			)*
		}
	};
}

// generated from https://docs.rs/serenity/0.8.9/src/serenity/client/event_handler.rs.html#12-314
// with help from vscode multiline editing and some manual cleanup
event! {
    cache_ready => CacheReady { guilds: Vec<GuildId> },
    channel_create => ChannelCreate { channel: Arc<RwLock<GuildChannel>> },
    category_create => CategoryCreate { category: Arc<RwLock<ChannelCategory>> },
    category_delete => CategoryDelete { category: Arc<RwLock<ChannelCategory>> },
    private_channel_create => PrivateChannelCreate { channel: Arc<RwLock<PrivateChannel>> },
    channel_delete => ChannelDelete { channel: Arc<RwLock<GuildChannel>> },
    channel_pins_update => ChannelPinsUpdate { pin: ChannelPinsUpdateEvent },
    channel_recipient_addition => ChannelRecipientAddition { group_id: ChannelId, user: User },
    channel_recipient_removal => ChannelRecipientRemoval { group_id: ChannelId, user: User },
    channel_update => ChannelUpdate { old: Option<Channel>, new: Channel },
    guild_ban_addition => GuildBanAddition { guild_id: GuildId, banned_user: User },
    guild_ban_removal => GuildBanRemoval { guild_id: GuildId, unbanned_user: User },
    guild_create => GuildCreate { guild: Guild, is_new: bool },
    guild_delete => GuildDelete { incomplete: PartialGuild, full: Option<Arc<RwLock<Guild>>> },
    guild_emojis_update => GuildEmojisUpdate { guild_id: GuildId, current_state: HashMap<EmojiId, Emoji> },
    guild_integrations_update => GuildIntegrationsUpdate { guild_id: GuildId },
    guild_member_addition => GuildMemberAddition { guild_id: GuildId, new_member: Member },
    guild_member_removal => GuildMemberRemoval { guild: GuildId, user: User, member_data_if_available: Option<Member> },
    guild_member_update => GuildMemberUpdate { old_if_available: Option<Member>, new: Member },
    guild_members_chunk => GuildMembersChunk { guild_id: GuildId, offline_members: HashMap<UserId, Member> },
    guild_role_create => GuildRoleCreate { guild_id: GuildId, new: Role },
    guild_role_delete => GuildRoleDelete { guild_id: GuildId, removed_role_id: RoleId, removed_role_data_if_available: Option<Role> },
    guild_role_update => GuildRoleUpdate { guild_id: GuildId, old_data_if_available: Option<Role>, new: Role },
    guild_unavailable => GuildUnavailable { guild_id: GuildId },
    guild_update => GuildUpdate { old_data_if_available: Option<Arc<RwLock<Guild>>>, new_but_incomplete: PartialGuild },
    message => Message { new_message: Message },
    message_delete => MessageDelete { channel_id: ChannelId, deleted_message_id: MessageId },
    message_delete_bulk => MessageDeleteBulk { channel_id: ChannelId, multiple_deleted_messages_ids: Vec<MessageId> },
    message_update => MessageUpdate { old_if_available: Option<Message>, new: Option<Message>, event: MessageUpdateEvent },
    reaction_add => ReactionAdd { add_reaction: Reaction },
    reaction_remove => ReactionRemove { removed_reaction: Reaction },
    reaction_remove_all => ReactionRemoveAll { channel_id: ChannelId, removed_from_message_id: MessageId },
    presence_replace => PresenceReplace { presences: Vec<Presence> },
    presence_update => PresenceUpdate { new_data: PresenceUpdateEvent },
    ready => Ready { data_about_bot: Ready },
    resume => Resume { event: ResumedEvent },
    shard_stage_update => ShardStageUpdate { event: ShardStageUpdateEvent },
    typing_start => TypingStart { event: TypingStartEvent },
    unknown => Unknown { name: String, raw: Value },
    user_update => UserUpdate { old_data: CurrentUser, new: CurrentUser },
    voice_server_update => VoiceServerUpdate { event: VoiceServerUpdateEvent },
    voice_state_update => VoiceStateUpdate { guild_id: Option<GuildId>, old: Option<VoiceState>, new: VoiceState },
    webhook_update => WebhookUpdate { guild_id: GuildId, belongs_to_channel_id: ChannelId },
}
