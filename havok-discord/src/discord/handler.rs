use super::command::alias::AliasContainer;
use serenity::async_trait;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::prelude::GuildId;
use serenity::prelude::Context;
use serenity::prelude::EventHandler;
use tracing::error;
use tracing::info;
use tracing_unwrap::OptionExt;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as `{}`", ready.user.name);
        for guild in ready.guilds {
            let guild_id = guild.id;
            if guild_id != GuildId(0) {
                {
                    let mut data = ctx.data.write().await;
                    let all = data.get_mut::<AliasContainer>().unwrap_or_log();
                    if let Err(why) = all.load_alias_data(*guild_id.as_u64()) {
                        error!("Error loading aliases `{}`", why)
                    }
                }
            }
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}
