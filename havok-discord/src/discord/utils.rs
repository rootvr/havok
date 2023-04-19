use serenity::model::prelude::Message;
use serenity::prelude::Context;

pub(crate) const PREFIX_SIGIL: &str = "/";

pub(crate) fn get_chat_id(msg: &Message) -> u64 {
    match msg.guild_id {
        Some(id) => *id.as_u64(),
        None => *msg.channel_id.as_u64(),
    }
}

pub(crate) async fn get_user_name(ctx: &Context, msg: &Message) -> String {
    match msg.guild_id {
        Some(id) => msg
            .author
            .nick_in(ctx, id)
            .await
            .unwrap_or_else(|| msg.author.name.to_owned()),
        None => msg.author.name.to_owned(),
    }
}

#[inline]
pub(crate) async fn send_reply(
    ctx: &Context,
    msg: &Message,
    reply: &str,
) -> Result<Message, serenity::Error> {
    msg.reply_ping(ctx, reply).await
}
