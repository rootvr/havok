pub(crate) mod map;
use map::RollMap;

pub(crate) mod utils;
use utils::check_critics;
use utils::parse_args;
use utils::react_to_critic;
use utils::search_critics;
use utils::solve;

use crate::discord::utils::get_user_name;
use crate::discord::utils::send_reply;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing_unwrap::OptionExt;

#[group]
#[description = "Roll expression solving group"]
#[commands(roll, reroll)]
struct Roll;

#[command]
#[aliases("r")]
#[min_args(1)]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (to_send, critics) = parse_args(ctx, msg, args).await;
    let sent = send_reply(ctx, msg, &to_send).await?;
    react_to_critic(ctx, &sent, critics).await?;
    Ok(())
}

#[command]
#[aliases("rr")]
async fn reroll(ctx: &Context, msg: &Message) -> CommandResult {
    let (to_send, critics) = {
        let solver = {
            let mut data = ctx.data.write().await;
            let roll_map = data.get_mut::<RollMap>().unwrap_or_log();
            roll_map.remove(&msg.author.to_string())
        };
        match solver {
            Some(solver) => {
                let query = solver.as_str().to_string();
                match solve(ctx, msg, solver).await {
                    Ok(result) => {
                        let critics = search_critics(&result);
                        (
                            format!("**rerolling** `{}`\n{}", query, result),
                            check_critics(critics),
                        )
                    }
                    Err(error) => (error, None),
                }
            }
            None => (
                format!(
                    "**error** *no previous rolls for* **{}**",
                    get_user_name(ctx, msg).await
                ),
                None,
            ),
        }
    };
    let sent = send_reply(ctx, msg, &to_send).await?;
    react_to_critic(ctx, &sent, critics).await?;
    Ok(())
}
