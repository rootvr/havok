use crate::discord::send_msg;
use crate::discord::ShardManagerContainer;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing::info;

#[group]
#[commands(quit)]
struct Owner;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Received `{:?}`", msg.content);
    let data = ctx.data.read().await;
    if let Some(shard_manager) = data.get::<ShardManagerContainer>() {
        send_msg(ctx, msg, "**info** *shutting down*").await?;
        // TODO(resu): persist aliases
        shard_manager.lock().await.shutdown_all().await;
    } else {
        send_msg(ctx, msg, "**error** *while getting shard manager*").await?;
    }
    Ok(())
}
