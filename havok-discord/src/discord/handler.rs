use crate::command::alias::map::AliasMap;
use serenity::async_trait;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::prelude::GuildId;
use serenity::prelude::Context;
use serenity::prelude::EventHandler;
use tracing::debug;
use tracing::info;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as `{}`", ready.user.name);
        for guild in ready.guilds {
            let guild_id = guild.id;
            if guild_id != GuildId(0) {
                {
                    let mut data = ctx.data.write().await;
                    let all = data.get_mut::<AliasMap>().unwrap_or_log();
                    all.load_alias_data(*guild_id.as_u64())
                        .expect_or_log("Error loading aliases `{}`");
                }
            }
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        debug!("Resumed");
    }
}
