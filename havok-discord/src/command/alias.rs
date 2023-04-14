pub mod map;
use map::AliasMap;

pub mod model;

use crate::discord::utils::send_reply;
use itertools::Itertools;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing::debug;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

fn get_chat_id(msg: &Message) -> u64 {
    match msg.guild_id {
        Some(id) => *id.as_u64(),
        None => *msg.channel_id.as_u64(),
    }
}

pub async fn get_user_name(ctx: &Context, msg: &Message) -> String {
    match msg.guild_id {
        Some(id) => msg
            .author
            .nick_in(ctx, id)
            .await
            .unwrap_or_else(|| msg.author.name.to_owned()),
        None => msg.author.name.to_owned(),
    }
}

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

#[group]
#[prefix = "alias"]
#[description = "Alias management group"]
#[commands(list_alias, set_user_alias, save_alias, load_alias)]
struct Alias;

#[command]
#[aliases("ls", "list")]
#[min_args(0)]
async fn list_alias(ctx: &Context, msg: &Message) -> CommandResult {
    debug!("Received `{:?}`", msg.content);
    let send = {
        let data = ctx.data.read().await;
        let all = data.get::<AliasMap>().unwrap_or_log();
        let (user_aliases, global_aliases) =
            all.list_alias(get_chat_id(msg), *msg.author.id.as_u64());
        let name = get_user_name(ctx, msg).await;
        format!(
            "**aliases**\n**global** *aliases*:\n{}\n\n**{}**'s *aliases*:\n{}",
            if !global_aliases.is_empty() {
                global_aliases.iter().format("\n").to_string()
            } else {
                "*empty*".to_owned()
            },
            name,
            if !user_aliases.is_empty() {
                user_aliases.iter().format("\n").to_string()
            } else {
                "*empty*".to_owned()
            }
        )
    };
    send_reply(ctx, msg, &send).await?;
    Ok(())
}

#[command]
#[aliases("su", "set")]
#[min_args(2)]
async fn set_user_alias(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    debug!("Received `{:?}`", msg.content);
    let send = {
        let alias = args.single::<String>().unwrap_or_log();
        let command = args.rest().to_string();
        let mut data = ctx.data.write().await;
        let all = data.get_mut::<AliasMap>().unwrap_or_log();
        all.set_user_alias(
            alias,
            command,
            get_chat_id(msg),
            *msg.author.id.as_u64(),
            &get_user_name(ctx, msg).await,
        )
    };
    send_reply(ctx, msg, &send).await?;
    Ok(())
}

#[command]
#[aliases("sv", "save")]
#[max_args(0)]
async fn save_alias(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    debug!("Received `{:?}`", msg.content);
    let send = {
        let data = ctx.data.read().await;
        let all = data.get::<AliasMap>().unwrap_or_log();
        all.save_alias_data(get_chat_id(msg))?
    };
    send_reply(ctx, msg, send).await?;
    Ok(())
}

#[command]
#[aliases("ld", "load")]
#[max_args(0)]
async fn load_alias(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    debug!("Received `{:?}`", msg.content);
    let send = {
        let mut data = ctx.data.write().await;
        let all = data.get_mut::<AliasMap>().unwrap_or_log();
        all.load_alias_data(get_chat_id(msg))?
    };
    send_reply(ctx, msg, send).await?;
    Ok(())
}
