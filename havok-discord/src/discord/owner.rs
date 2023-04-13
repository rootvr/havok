use super::send;
use super::ShardManagerContainer;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;

#[group]
#[commands(quit)]
struct Owner;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    if let Some(shard_manager) = data.get::<ShardManagerContainer>() {
        send(ctx, msg, "shutting down").await?;
        shard_manager.lock().await.shutdown_all().await;
    } else {
        send(ctx, msg, "error while getting shard manager").await?;
    }
    Ok(())
}
