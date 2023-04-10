use super::send;
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
async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if !args.is_empty() {
        let r = Solver::new(args.rest())
            .unwrap_or_log()
            .solve()
            .unwrap_or_log();
        send(ctx, msg, &r.to_string()).await?;
    }
    Ok(())
}
