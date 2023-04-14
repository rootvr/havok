use crate::discord::map::ShardManagerMap;
use crate::discord::utils::send_reply;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing::debug;

#[group]
#[description = "General management group"]
#[commands(ping, quit)]
struct Meta;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    debug!("Received `{:?}`", msg.content);
    send_reply(ctx, msg, "**info** *pong*").await?;
    Ok(())
}

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    debug!("Received `{:?}`", msg.content);
    let data = ctx.data.read().await;
    if let Some(shard_manager) = data.get::<ShardManagerMap>() {
        send_reply(ctx, msg, "**info** *shutting down*").await?;
        // TODO(resu): persist aliases
        shard_manager.lock().await.shutdown_all().await;
    } else {
        send_reply(ctx, msg, "**error** *while getting shard manager*").await?;
    }
    Ok(())
}
