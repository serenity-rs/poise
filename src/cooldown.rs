//! Infrastructure for command cooldowns

use crate::serenity_prelude as serenity;
// I usually don't really do imports, but these are very convenient
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// The starting value that gets assigned when inserted into [`CooldownTracker`]
const STARTING_BURST_AMOUNT: u64 = 1;
/// The default burst amount if it is not provided in [`CooldownConfig`]
const DEFAULT_BURST_AMOUNT: u64 = 1;

/// Subset of [`crate::Context`] so that [`Cooldowns`] can be used without requiring a full [Context](`crate::Context`)
/// (ie from within an `event_handler`)
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct CooldownContext {
    /// The user associated with this request
    pub user_id: serenity::UserId,
    /// The guild this request originated from or `None`
    pub guild_id: Option<serenity::GuildId>,
    /// The channel associated with this request
    pub channel_id: serenity::ChannelId,
}

/// Configuration struct for [`Cooldowns`]
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct CooldownConfig {
    /// This cooldown operates on a global basis
    pub global: Option<Duration>,
    /// This is how many operations can be invoked within the cooldown period on a global basis
    pub global_burst_amount: Option<u64>,
    /// This cooldown operates on a per-user basis
    pub user: Option<Duration>,
    /// This is how many operations can be invoked within the cooldown period on a per-user basis
    pub user_burst_amount: Option<u64>,
    /// This cooldown operates on a per-guild basis
    pub guild: Option<Duration>,
    /// This is how many operations can be invoked within the cooldown period on a per-guild basis
    pub guild_burst_amount: Option<u64>,
    /// This cooldown operates on a per-channel basis
    pub channel: Option<Duration>,
    /// This is how many operations can be invoked within the cooldown period on a per-channel basis
    pub channel_burst_amount: Option<u64>,
    /// This cooldown operates on a per-member basis
    pub member: Option<Duration>,
    /// This is how many operations can be invoked within the cooldown period on a per-member basis
    pub member_burst_amount: Option<u64>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

/// Tracks all types of cooldowns for a single command
///
/// You probably don't need to use this directly. `#[poise::command]` automatically generates a
/// cooldown handler.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CooldownTracker {
    /// Stores the timestamp of the last global invocation
    global_invocation: Option<(Instant, u64)>,
    /// Stores the timestamps of the last invocation per user
    user_invocations: HashMap<serenity::UserId, (Instant, u64)>,
    /// Stores the timestamps of the last invocation per guild
    guild_invocations: HashMap<serenity::GuildId, (Instant, u64)>,
    /// Stores the timestamps of the last invocation per channel
    channel_invocations: HashMap<serenity::ChannelId, (Instant, u64)>,
    /// Stores the timestamps of the last invocation per member (user and guild)
    member_invocations: HashMap<(serenity::UserId, serenity::GuildId), (Instant, u64)>,
}

/// Possible types of command cooldowns.
///
/// Currently used for [CooldownTracker::set_last_invocation]
#[non_exhaustive]
pub enum CooldownType {
    /// A global cooldown that applies to all users, channels, and guilds.
    Global,
    /// A cooldown specific to individual users.
    User(serenity::UserId),
    /// A cooldown that applies to an entire guild.
    Guild(serenity::GuildId),
    /// A cooldown specific to individual channels.
    Channel(serenity::ChannelId),
    /// A cooldown specific to individual members within a guild.
    Member((serenity::UserId, serenity::GuildId)),
}

/// **Renamed to [`CooldownTracker`]**
pub use CooldownTracker as Cooldowns;

impl CooldownTracker {
    /// Create a new cooldown tracker
    pub fn new() -> Self {
        Self {
            global_invocation: None,
            user_invocations: HashMap::new(),
            guild_invocations: HashMap::new(),
            channel_invocations: HashMap::new(),
            member_invocations: HashMap::new(),
        }
    }

    /// Queries the cooldown buckets and checks if all cooldowns have expired and command
    /// execution may proceed. If not, Some is returned with the remaining cooldown
    pub fn remaining_cooldown(
        &self,
        ctx: CooldownContext,
        cooldown_durations: &CooldownConfig,
    ) -> Option<Duration> {
        let mut cooldown_data = vec![
            (
                cooldown_durations.global,
                cooldown_durations.global_burst_amount,
                self.global_invocation,
            ),
            (
                cooldown_durations.user,
                cooldown_durations.user_burst_amount,
                self.user_invocations.get(&ctx.user_id).copied(),
            ),
            (
                cooldown_durations.channel,
                cooldown_durations.channel_burst_amount,
                self.channel_invocations.get(&ctx.channel_id).copied(),
            ),
        ];

        if let Some(guild_id) = ctx.guild_id {
            cooldown_data.push((
                cooldown_durations.guild,
                cooldown_durations.guild_burst_amount,
                self.guild_invocations.get(&guild_id).copied(),
            ));
            cooldown_data.push((
                cooldown_durations.member,
                cooldown_durations.member_burst_amount,
                self.member_invocations
                    .get(&(ctx.user_id, guild_id))
                    .copied(),
            ));
        }

        cooldown_data
            .iter()
            .filter_map(|&(cooldown, burst_amount, last_invocation)| {
                let last_invocation = last_invocation?;
                if burst_amount.unwrap_or(DEFAULT_BURST_AMOUNT) > last_invocation.1 {
                    return None;
                }
                let duration_since = Instant::now().saturating_duration_since(last_invocation.0);
                let cooldown_left = cooldown?.checked_sub(duration_since)?;
                Some(cooldown_left)
            })
            .max()
    }

    /// Indicates that a command has been executed and all associated cooldowns should start running
    pub fn start_cooldown(&mut self, ctx: CooldownContext) {
        let now = Instant::now();

        self.global_invocation = Some((now, STARTING_BURST_AMOUNT));
        self.user_invocations.insert(ctx.user_id, (now, STARTING_BURST_AMOUNT));
        self.channel_invocations.insert(ctx.channel_id, (now, STARTING_BURST_AMOUNT));

        if let Some(guild_id) = ctx.guild_id {
            self.guild_invocations.insert(guild_id, (now, STARTING_BURST_AMOUNT));
            self.member_invocations
                .insert((ctx.user_id, guild_id), (now, STARTING_BURST_AMOUNT));
        }
    }

    /// Sets the last invocation for the specified cooldown bucket.
    ///
    /// This function is not usually needed for regular usage. It was added to allow for extra
    /// flexibility in cases where you might want to shorten or lengthen a cooldown after
    /// invocation.
    pub fn set_last_invocation(&mut self, cooldown_type: CooldownType, instant: Instant) {
        // FIXME: decide whether burst amounts should just be set strictly to 1 or if the old value
        // should cascade
        match cooldown_type {
            CooldownType::Global => {
                self.global_invocation = Some((
                    instant,
                    self.global_invocation
                        .map_or(STARTING_BURST_AMOUNT, |(_, burst_amount)| burst_amount),
                ))
            }
            CooldownType::User(user_id) => {
                self.user_invocations.insert(
                    user_id,
                    (
                        instant,
                        self.global_invocation
                            .map_or(STARTING_BURST_AMOUNT, |(_, burst_amount)| burst_amount),
                    ),
                );
            }
            CooldownType::Guild(guild_id) => {
                self.guild_invocations.insert(
                    guild_id,
                    (
                        instant,
                        self.global_invocation
                            .map_or(STARTING_BURST_AMOUNT, |(_, burst_amount)| burst_amount),
                    ),
                );
            }
            CooldownType::Channel(channel_id) => {
                self.channel_invocations.insert(
                    channel_id,
                    (
                        instant,
                        self.global_invocation
                            .map_or(STARTING_BURST_AMOUNT, |(_, burst_amount)| burst_amount),
                    ),
                );
            }
            CooldownType::Member(member) => {
                self.member_invocations.insert(
                    member,
                    (
                        instant,
                        self.global_invocation
                            .map_or(STARTING_BURST_AMOUNT, |(_, burst_amount)| burst_amount),
                    ),
                );
            }
        }
    }

    /// Increments burst counter by 1, and otherwise resets the cooldown if the initial cooldown
    /// window has passed.
    pub fn increment_usage(&mut self, ctx: CooldownContext, config: &CooldownConfig) {
        let now = Instant::now();
        self.global_invocation = Some(
            self.global_invocation
                .and_then(|(window, burst_amount)| {
                    if let Some(global_cooldown) = config.global {
                        let duration_since = now.saturating_duration_since(window);
                        if duration_since <= global_cooldown {
                            return Some((window, burst_amount + 1));
                        }
                    }
                    None
                })
                .unwrap_or((now, STARTING_BURST_AMOUNT)),
        );
        self.user_invocations
            .entry(ctx.user_id)
            .and_modify(|(window, burst_amount)| {
                if let Some(user_cooldown) = config.user {
                    let duration_since = now.saturating_duration_since(*window);
                    if duration_since <= user_cooldown {
                        *burst_amount += 1;
                        return;
                    }
                }
                *window = now;
                *burst_amount = STARTING_BURST_AMOUNT;
            })
            .or_insert((now, STARTING_BURST_AMOUNT));
        self.channel_invocations
            .entry(ctx.channel_id)
            .and_modify(|(window, burst_amount)| {
                if let Some(channel_cooldown) = config.channel {
                    let duration_since = now.saturating_duration_since(*window);
                    if duration_since <= channel_cooldown {
                        *burst_amount += 1;
                        return;
                    }
                }
                *window = now;
                *burst_amount = STARTING_BURST_AMOUNT;
            })
            .or_insert((now, STARTING_BURST_AMOUNT));

        if let Some(guild_id) = ctx.guild_id {
            self.guild_invocations
                .entry(guild_id)
                .and_modify(|(window, burst_amount)| {
                    if let Some(guild_cooldown) = config.guild {
                        let duration_since = now.saturating_duration_since(*window);
                        if duration_since <= guild_cooldown {
                            *burst_amount += 1;
                            return;
                        }
                    }
                    *window = now;
                    *burst_amount = STARTING_BURST_AMOUNT;
                })
                .or_insert((now, STARTING_BURST_AMOUNT));
            self.member_invocations
                .entry((ctx.user_id, guild_id))
                .and_modify(|(window, burst_amount)| {
                    if let Some(member_cooldown) = config.member {
                        let duration_since = now.saturating_duration_since(*window);
                        if duration_since <= member_cooldown {
                            *burst_amount += 1;
                            return;
                        }
                    }
                    *window = now;
                    *burst_amount = STARTING_BURST_AMOUNT;
                })
                .or_insert((now, STARTING_BURST_AMOUNT));
        }
    }
}

impl<'a> From<&'a serenity::Message> for CooldownContext {
    fn from(message: &'a serenity::Message) -> Self {
        Self {
            user_id: message.author.id,
            channel_id: message.channel_id,
            guild_id: message.guild_id,
        }
    }
}

#[cfg(test)]
mod test {
    use ::serenity::all::{ChannelId, GuildId, UserId};

    use super::*;

    #[test]
    fn start_cooldown_triggers_guild_cooldown() {
        let config = CooldownConfig {
            global: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
    }

    #[test]
    fn cooldown_resets_after_window_expires() {
        let config = CooldownConfig {
            global: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let tracker = CooldownTracker {
            global_invocation: Some((Instant::now() - Duration::from_secs(1), 1)),
            ..Default::default()
        };
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());
    }

    #[tokio::test]
    async fn basic_global_cooldown() {
        let config = CooldownConfig {
            global: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx.clone(), &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
        tokio::time::sleep(cooldown).await;
        assert!(tracker.remaining_cooldown(ctx, &config).is_none());
    }

    #[test]
    fn global_cooldown_affects_other_users() {
        let config = CooldownConfig {
            global: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));

        let other_user_ctx = CooldownContext {
            user_id: UserId::from(54321),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        let cooldown = tracker.remaining_cooldown(other_user_ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
    }

    #[tokio::test]
    async fn basic_user_cooldown() {
        let config = CooldownConfig {
            user: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx.clone(), &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
        tokio::time::sleep(cooldown).await;
        assert!(tracker.remaining_cooldown(ctx, &config).is_none());
    }

    #[test]
    fn user_cooldown_does_not_affect_other_users() {
        let config = CooldownConfig {
            user: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));

        let other_user_ctx = CooldownContext {
            user_id: UserId::from(54321),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker
            .remaining_cooldown(other_user_ctx, &config)
            .is_none());

        let same_user_different_channel_ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(9876),
        };

        let cooldown = tracker.remaining_cooldown(same_user_different_channel_ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
    }

    #[tokio::test]
    async fn basic_channel_cooldown() {
        let config = CooldownConfig {
            channel: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx.clone(), &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
        tokio::time::sleep(cooldown).await;
        assert!(tracker.remaining_cooldown(ctx, &config).is_none())
    }

    #[test]
    fn channel_cooldown_affects_one_channel() {
        let config = CooldownConfig {
            channel: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));

        let other_user_ctx = CooldownContext {
            user_id: UserId::from(54321),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        let cooldown = tracker.remaining_cooldown(other_user_ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));

        let same_user_different_channel_ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(9876),
        };

        assert!(tracker
            .remaining_cooldown(same_user_different_channel_ctx, &config)
            .is_none());
    }

    #[tokio::test]
    async fn basic_guild_cooldown() {
        let config = CooldownConfig {
            guild: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx.clone(), &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
        tokio::time::sleep(cooldown).await;
        assert!(tracker.remaining_cooldown(ctx, &config).is_none());
    }

    #[test]
    fn guild_cooldown_affects_one_guild() {
        let config = CooldownConfig {
            guild: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(67890),
        };

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));

        let other_user_ctx = CooldownContext {
            user_id: UserId::from(54321),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(67890),
        };

        let cooldown = tracker.remaining_cooldown(other_user_ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));

        // This is not realistic since while guild id is different, the channel id is the same, but
        // this is to demostrate the guild id affects the cooldown.
        let same_user_different_guild_ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: Some(GuildId::from(420)),
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker
            .remaining_cooldown(same_user_different_guild_ctx, &config)
            .is_none());
    }

    #[tokio::test]
    async fn basic_member_cooldown() {
        let config = CooldownConfig {
            member: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx.clone(), &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
        tokio::time::sleep(cooldown).await;
        assert!(tracker.remaining_cooldown(ctx, &config).is_none());
    }

    #[test]
    fn member_cooldown_affects_one_member() {
        let config = CooldownConfig {
            member: Some(Duration::from_secs(1)),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(67890),
        };

        tracker.start_cooldown(ctx.clone());

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));

        let other_user_ctx = CooldownContext {
            user_id: UserId::from(54321),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(67890),
        };

        assert!(tracker
            .remaining_cooldown(other_user_ctx, &config)
            .is_none());

        let same_user_different_channel_ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(9876),
        };
        let cooldown = tracker.remaining_cooldown(same_user_different_channel_ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(1));
        assert!(cooldown > Duration::from_secs(0));
    }

    #[test]
    fn global_bursts_do_not_return_cooldown_burst_count_is_exceeded() {
        const BURST_AMOUNT: u64 = 10;
        let config = CooldownConfig {
            global: Some(Duration::from_secs(10)),
            global_burst_amount: Some(BURST_AMOUNT),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: None,
            channel_id: ChannelId::from(67890),
        };

        for _ in 0..BURST_AMOUNT {
            assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());

            tracker.increment_usage(ctx.clone(), &config);
        }

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(10));
        assert!(cooldown > Duration::from_secs(9));
    }

    #[test]
    fn member_bursts_do_not_return_cooldown_burst_count_is_exceeded() {
        const BURST_AMOUNT: u64 = 10;
        let config = CooldownConfig {
            member: Some(Duration::from_secs(10)),
            member_burst_amount: Some(BURST_AMOUNT),
            ..Default::default()
        };
        let mut tracker = CooldownTracker::default();
        let ctx = CooldownContext {
            user_id: UserId::from(12345),
            guild_id: Some(GuildId::from(1337)),
            channel_id: ChannelId::from(67890),
        };

        for _ in 0..BURST_AMOUNT {
            assert!(tracker.remaining_cooldown(ctx.clone(), &config).is_none());

            tracker.increment_usage(ctx.clone(), &config);
        }

        let cooldown = tracker.remaining_cooldown(ctx, &config);
        assert!(cooldown.is_some());
        let cooldown = cooldown.unwrap();
        assert!(cooldown < Duration::from_secs(10));
        assert!(cooldown > Duration::from_secs(9));
    }
}
