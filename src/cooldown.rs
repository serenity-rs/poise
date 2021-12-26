//! Infrastructure for command cooldowns

use crate::serenity_prelude as serenity;
// I usually don't really do imports, but these are very convenient
use crate::util::OrderedMap;
use std::time::{Duration, Instant};

/// Configuration struct for [`Cooldowns`]
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct CooldownConfig {
    /// This cooldown operates on a global basis
    pub global: Option<Duration>,
    /// This cooldown operates on a per-user basis
    pub user: Option<Duration>,
    /// This cooldown operates on a per-guild basis
    pub guild: Option<Duration>,
    /// This cooldown operates on a per-channel basis
    pub channel: Option<Duration>,
    /// This cooldown operates on a per-member basis
    pub member: Option<Duration>,
}

/// Handles cooldowns for a single command
///
/// You probably don't need to use this directly. `#[poise::command]` automatically generates a
/// cooldown handler.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cooldowns {
    /// Stores the cooldown durations
    cooldown: CooldownConfig,

    /// Stores the timestamp of the last global invocation
    global_invocation: Option<Instant>,
    /// Stores the timestamps of the last invocation per user
    user_invocations: OrderedMap<serenity::UserId, Instant>,
    /// Stores the timestamps of the last invocation per guild
    guild_invocations: OrderedMap<serenity::GuildId, Instant>,
    /// Stores the timestamps of the last invocation per channel
    channel_invocations: OrderedMap<serenity::ChannelId, Instant>,
    /// Stores the timestamps of the last invocation per member (user and guild)
    member_invocations: OrderedMap<(serenity::UserId, serenity::GuildId), Instant>,
}

impl Cooldowns {
    /// Create a new cooldown handler with the given cooldown durations
    pub fn new(config: CooldownConfig) -> Self {
        Self {
            cooldown: config,

            global_invocation: None,
            user_invocations: OrderedMap::new(),
            guild_invocations: OrderedMap::new(),
            channel_invocations: OrderedMap::new(),
            member_invocations: OrderedMap::new(),
        }
    }

    /// Queries the cooldown buckets and checks if all cooldowns have expired and command
    /// execution may proceed. If not, Some is returned with the remaining cooldown
    pub fn remaining_cooldown<U, E>(&self, ctx: crate::Context<'_, U, E>) -> Option<Duration> {
        let mut cooldown_data = vec![
            (self.cooldown.global, self.global_invocation),
            (
                self.cooldown.user,
                self.user_invocations.get(&ctx.author().id).copied(),
            ),
            (
                self.cooldown.channel,
                self.channel_invocations.get(&ctx.channel_id()).copied(),
            ),
        ];

        if let Some(guild_id) = ctx.guild_id() {
            cooldown_data.push((
                self.cooldown.guild,
                self.guild_invocations.get(&guild_id).copied(),
            ));
            cooldown_data.push((
                self.cooldown.member,
                self.member_invocations
                    .get(&(ctx.author().id, guild_id))
                    .copied(),
            ));
        }

        cooldown_data
            .iter()
            .filter_map(|&(cooldown, last_invocation)| {
                let duration_since = Instant::now().saturating_duration_since(last_invocation?);
                let cooldown_left = cooldown?.checked_sub(duration_since)?;
                Some(cooldown_left)
            })
            .max()
    }

    /// Indicates that a command has been executed and all associated cooldowns should start running
    pub fn start_cooldown<U, E>(&mut self, ctx: crate::Context<'_, U, E>) {
        let now = Instant::now();

        self.global_invocation = Some(now);
        self.user_invocations.insert(ctx.author().id, now);
        self.channel_invocations.insert(ctx.channel_id(), now);

        if let Some(guild_id) = ctx.guild_id() {
            self.guild_invocations.insert(guild_id, now);
            self.member_invocations
                .insert((ctx.author().id, guild_id), now);
        }
    }
}
