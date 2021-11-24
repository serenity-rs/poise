use crate::serenity_prelude as serenity;
// I usually don't do imports, but these are very convenient
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct Cooldowns {
    global_cooldown: Option<Duration>,
    global_invocation: Mutex<Option<Instant>>,
    user_cooldown: Option<Duration>,
    user_invocations: Mutex<HashMap<serenity::UserId, Instant>>,
    channel_cooldown: Option<Duration>,
    channel_invocations: Mutex<HashMap<serenity::ChannelId, Instant>>,
}

impl Cooldowns {
    pub fn new(
        global_cooldown: Option<Duration>,
        user_cooldown: Option<Duration>,
        channel_cooldown: Option<Duration>,
    ) -> Self {
        Self {
            global_cooldown,
            global_invocation: Mutex::new(None),
            user_cooldown,
            user_invocations: Mutex::new(HashMap::new()),
            channel_cooldown,
            channel_invocations: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_wait_time<U, E>(&self, ctx: crate::Context<'_, U, E>) -> Option<Duration> {
        let cooldown_data = &[
            (
                self.global_cooldown,
                *self.global_invocation.lock().unwrap(),
            ),
            (
                self.user_cooldown,
                self.user_invocations
                    .lock()
                    .unwrap()
                    .get(&ctx.author().id)
                    .copied(),
            ),
            (
                self.channel_cooldown,
                self.channel_invocations
                    .lock()
                    .unwrap()
                    .get(&ctx.channel_id())
                    .copied(),
            ),
        ];

        cooldown_data
            .iter()
            .filter_map(|&(cooldown, last_invocation)| {
                let duration_since = Instant::now().saturating_duration_since(last_invocation?);
                let cooldown_left = cooldown?.checked_sub(duration_since)?;
                Some(cooldown_left)
            })
            .max()
    }

    pub fn trigger<U, E>(&self, ctx: crate::Context<'_, U, E>) {
        let now = Instant::now();

        *self.global_invocation.lock().unwrap() = Some(now);
        self.user_invocations
            .lock()
            .unwrap()
            .insert(ctx.author().id, now);
        self.channel_invocations
            .lock()
            .unwrap()
            .insert(ctx.channel_id(), now);
    }
}
