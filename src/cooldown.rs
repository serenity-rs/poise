use crate::serenity_prelude as serenity;
// I usually don't do imports, but these are very convenient
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct CooldownConfig {
    pub global: Option<Duration>,
    pub user: Option<Duration>,
    pub guild: Option<Duration>,
    pub channel: Option<Duration>,
    pub member: Option<Duration>,
}

pub struct Cooldowns {
    cooldown: CooldownConfig,

    global_invocation: Option<Instant>,
    user_invocations: HashMap<serenity::UserId, Instant>,
    guild_invocations: HashMap<serenity::GuildId, Instant>,
    channel_invocations: HashMap<serenity::ChannelId, Instant>,
    member_invocations: HashMap<(serenity::UserId, serenity::GuildId), Instant>,
}

impl Cooldowns {
    pub fn new(config: CooldownConfig) -> Self {
        Self {
            cooldown: config,

            global_invocation: None,
            user_invocations: HashMap::new(),
            guild_invocations: HashMap::new(),
            channel_invocations: HashMap::new(),
            member_invocations: HashMap::new(),
        }
    }

    pub fn get_wait_time<U, E>(&self, ctx: crate::Context<'_, U, E>) -> Option<Duration> {
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
