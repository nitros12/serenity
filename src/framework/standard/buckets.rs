use chrono::Utc;
use client::Context;
use model::id::{ChannelId, GuildId, UserId};
use std::{
    collections::HashMap,
    default::Default
};

#[cfg(feature = "cache")]
type Check = Fn(&mut Context, Option<GuildId>, ChannelId, UserId) -> bool + Send + Sync + 'static;

#[cfg(not(feature = "cache"))]
type Check = Fn(&mut Context, ChannelId, UserId) -> bool + 'static;

pub(crate) struct Ratelimit {
    pub delay: i64,
    pub limit: Option<(i64, i32)>,
}

#[derive(Default)]
pub(crate) struct GuildRatelimit {
    pub last_time: i64,
    pub set_time: i64,
    pub tickets: i32,
}

pub(crate) struct Bucket {
    pub ratelimit: Ratelimit,
    pub guilds: HashMap<u64, GuildRatelimit>,
    pub check: Option<Box<Check>>,
}

impl Bucket {
    pub fn take(&mut self, guild_id: u64) -> i64 {
        let time = Utc::now().timestamp();
        let guild = self.guilds
            .entry(guild_id)
            .or_insert_with(GuildRatelimit::default);

        if let Some((timespan, limit)) = self.ratelimit.limit {
            if (guild.tickets + 1) > limit {
                if time < (guild.set_time + timespan) {
                    return (guild.set_time + timespan) - time;
                } else {
                    guild.tickets = 0;
                    guild.set_time = time;
                }
            }
        }

        if time < guild.last_time + self.ratelimit.delay {
            (guild.last_time + self.ratelimit.delay) - time
        } else {
            guild.tickets += 1;
            guild.last_time = time;

            0
        }
    }
}
