use crate::discord::err_msg;
use crate::discord::send_msg;
use havok_lib::roll::Kind;
use havok_lib::solver::Solver;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing_unwrap::ResultExt;

#[group]
#[commands(roll)]
struct Havok;

#[command]
#[aliases("r")]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if !args.is_empty() {
        let r = Solver::new(args.rest()).unwrap_or_log().solve();
        match r {
            Ok(r) => match r.get_result() {
                Kind::Single(_) => {
                    send_msg(ctx, msg, &format!("**single result**\n{}", r)).await?;
                }
                Kind::Multi(_) => {
                    send_msg(ctx, msg, &format!("**multi result**\n{}", r)).await?;
                }
            },
            Err(e) => {
                send_msg(ctx, msg, &err_msg(e)).await?;
            }
        }
    }
    Ok(())
}
