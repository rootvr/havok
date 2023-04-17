pub mod map;
use map::AliasMap;

pub mod model;

pub mod utils;

use crate::discord::utils::get_chat_id;
use crate::discord::utils::get_user_name;
use crate::discord::utils::send_reply;
use itertools::Itertools;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

#[group]
#[prefix = "alias"]
#[description = "Alias management group"]
#[commands(list, save, load, set, del, clear, setg, delg, clearg)]
struct Alias;

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let to_send = {
        let data = ctx.data.read().await;
        let alias_map = data.get::<AliasMap>().unwrap_or_log();
        let (user_defs, global_defs) =
            alias_map.list_alias(get_chat_id(msg), *msg.author.id.as_u64());
        let name = get_user_name(ctx, msg).await;
        format!(
            "**aliases**\n{}{}",
            global_defs
                .iter()
                .map(|s| format!("*global* {}\n", s))
                .format(""),
            user_defs
                .iter()
                .map(|s| format!("*{}* {}\n", name, s))
                .format("")
        )
    };
    send_reply(ctx, msg, &to_send).await?;
    Ok(())
}

#[command]
async fn save(ctx: &Context, msg: &Message) -> CommandResult {
    let to_send = {
        let data = ctx.data.read().await;
        let alias_map = data.get::<AliasMap>().unwrap_or_log();
        alias_map.save_alias_data(get_chat_id(msg))?
    };
    send_reply(ctx, msg, to_send).await?;
    Ok(())
}

#[command]
async fn load(ctx: &Context, msg: &Message) -> CommandResult {
    let to_send = {
        let mut data = ctx.data.write().await;
        let alias_map = data.get_mut::<AliasMap>().unwrap_or_log();
        alias_map.load_alias_data(get_chat_id(msg))?
    };
    send_reply(ctx, msg, to_send).await?;
    Ok(())
}

#[command]
#[min_args(2)]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let to_send = {
        let alias = args.single::<String>().unwrap_or_log();
        let command = args.rest().to_string();
        let mut data = ctx.data.write().await;
        let alias_map = data.get_mut::<AliasMap>().unwrap_or_log();
        alias_map.set_user_alias(
            alias,
            command,
            get_chat_id(msg),
            *msg.author.id.as_u64(),
            &get_user_name(ctx, msg).await,
        )
    };
    send_reply(ctx, msg, &to_send).await?;
    Ok(())
}

#[command]
#[min_args(1)]
async fn del(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let to_send = {
        let alias = args.single::<String>().unwrap_or_log();
        let mut data = ctx.data.write().await;
        let alias_map = data.get_mut::<AliasMap>().unwrap_or_log();
        alias_map.del_user_alias(&alias, get_chat_id(msg), *msg.author.id.as_u64())
    };
    send_reply(ctx, msg, &to_send).await?;
    Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    let to_send = {
        let mut data = ctx.data.write().await;
        let alias_map = data.get_mut::<AliasMap>().unwrap_or_log();
        alias_map.clear_user_aliases(get_chat_id(msg), *msg.author.id.as_u64())
    };
    send_reply(ctx, msg, to_send).await?;
    Ok(())
}

#[command]
#[min_args(2)]
async fn setg(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let to_send = {
        let alias = args.single::<String>().unwrap_or_log();
        let command = args.rest().to_string();
        let mut data = ctx.data.write().await;
        let alias_map = data.get_mut::<AliasMap>().unwrap_or_log();
        alias_map.set_global_alias(alias, command, get_chat_id(msg))
    };
    send_reply(ctx, msg, &to_send).await?;
    Ok(())
}

#[command]
#[min_args(1)]
async fn delg(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let to_send = {
        let mut data = ctx.data.write().await;
        let alias_map = data.get_mut::<AliasMap>().unwrap();
        alias_map.del_global_alias(args.rest(), get_chat_id(msg))
    };
    send_reply(ctx, msg, &to_send).await?;
    Ok(())
}

#[command]
async fn clearg(ctx: &Context, msg: &Message) -> CommandResult {
    let to_send = {
        let mut data = ctx.data.write().await;
        let alias_map = data.get_mut::<AliasMap>().unwrap();
        alias_map.clear_aliases(get_chat_id(msg))
    };
    send_reply(ctx, msg, to_send).await?;
    Ok(())
}
