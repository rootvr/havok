use serenity::model::prelude::Message;
use serenity::prelude::Context;

// TODO(resu): Make this dynamic
pub const PREFIX_SIGIL: &str = "!";

#[inline]
pub async fn send_reply(
    ctx: &Context,
    msg: &Message,
    reply: &str,
) -> Result<Message, serenity::Error> {
    msg.reply_ping(ctx, reply).await
}
