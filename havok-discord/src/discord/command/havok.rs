use crate::discord::send_msg;
use havok_lib::error::Error::Other;
use havok_lib::error::Error::Pest;
use havok_lib::solver::Solver;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing_unwrap::ResultExt;

fn err_msg(err: havok_lib::error::Error) -> String {
    match err {
        Pest(_) => format!("**error**\n```{}\n```", err),
        Other(err) => format!("**error** *{}*", err),
    }
}

#[group]
#[commands(roll)]
struct Havok;

#[command]
#[aliases("r")]
#[min_args(1)]
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if !args.is_empty() {
        match Solver::new(args.rest()).unwrap_or_log().solve() {
            Ok(r) => send_msg(ctx, msg, &format!("**roll**\n{}", r)).await?,
            Err(e) => send_msg(ctx, msg, &err_msg(e)).await?,
        };
    }
    Ok(())
}
