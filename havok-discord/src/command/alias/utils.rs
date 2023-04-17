use super::AliasMap;
use serenity::framework::standard::Args;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing_unwrap::OptionExt;

pub async fn parse_alias(
    ctx: &Context,
    _msg: &Message,
    args: Args,
) -> Result<(String, bool), String> {
    let data = ctx.data.read().await;
    let _ /*all*/ = data.get::<AliasMap>().unwrap_or_log();
    // all.expand_alias(args.rest(), get_chat_id(msg), *msg.author.id.as_u64(), true)
    Ok((args.rest().to_string(), false))
}
