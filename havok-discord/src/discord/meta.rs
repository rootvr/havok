use super::send;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing::info;

#[group]
#[commands(ping)]
struct Meta;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Received `{:?}`", msg.content);
    send(ctx, msg, "pong").await?;
    Ok(())
}
