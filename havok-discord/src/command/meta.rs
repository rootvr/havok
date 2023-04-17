use crate::command::alias::map::AliasMap;
use crate::discord::map::ShardManagerMap;
use crate::discord::utils::send_reply;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing_unwrap::OptionExt;

#[group]
#[description = "General management group"]
#[commands(ping, quit)]
struct Meta;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    send_reply(ctx, msg, "**info** *pong*").await?;
    Ok(())
}

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let shard_manager = data
        .get::<ShardManagerMap>()
        .expect_or_log("unable to get shard manager");
    send_reply(ctx, msg, "**info** *shutting down*").await?;
    let data = ctx.data.read().await;
    let all = data.get::<AliasMap>().unwrap_or_log();
    all.save_all();
    shard_manager.lock().await.shutdown_all().await;
    Ok(())
}
