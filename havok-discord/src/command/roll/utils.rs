use super::map::RollMap;
use crate::command::alias::parse_alias;
use havok_lib::dice::Critic;
use havok_lib::error::Error;
use havok_lib::roll;
use havok_lib::roll::history::History;
use havok_lib::solver::Solver;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::prelude::ReactionType;
use serenity::prelude::Context;
use std::borrow::Cow;
use std::collections::HashSet;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

const TWEMOJI_NOT: &str = "🤨";
const TWEMOJI_MIN: &str = "🥶";
const TWEMOJI_MAX: &str = "🤩";

fn format_havok_error(error: Error) -> String {
    match error {
        Error::Pest(_) => format!("**error**\n```{}\n```", error),
        Error::Other(error) => format!("**error** *{}*", error),
    }
}

pub async fn react_to_critic(
    ctx: &Context,
    msg: &Message,
    critics: Option<HashSet<Critic>>,
) -> CommandResult {
    if let Some(critics) = critics {
        for critic in critics.iter() {
            let twemoji = match critic {
                Critic::Not => ReactionType::Unicode(TWEMOJI_NOT.to_string()),
                Critic::Min => ReactionType::Unicode(TWEMOJI_MIN.to_string()),
                Critic::Max => ReactionType::Unicode(TWEMOJI_MAX.to_string()),
            };
            msg.react(ctx, twemoji).await?;
        }
    }
    Ok(())
}

pub async fn parse_args(
    ctx: &Context,
    msg: &Message,
    args: Args,
) -> (String, Option<HashSet<Critic>>) {
    // TODO(resu): expand aliases here
    let input = parse_alias(ctx, msg, args).await;
    let (input, has_alias) = match input {
        Ok(input) => input,
        Err(error) => return (error, None),
    };
    let alias = if has_alias {
        Cow::Owned(format!("*alias* `{}`\n", input))
    } else {
        Cow::Borrowed("")
    };
    match solve_expr(ctx, msg, &input).await {
        Ok(result) => {
            let critics = search_critics(&result);
            let result = result.to_string();
            (
                format!("**rolling** {}\n{}", alias, result),
                check_critics(critics),
            )
        }
        Err(error) => (error, None),
    }
}

async fn solve_expr(ctx: &Context, msg: &Message, input: &str) -> Result<roll::Result, String> {
    solve(ctx, msg, Solver::new(input).unwrap_or_log()).await
}

pub async fn solve(ctx: &Context, msg: &Message, solver: Solver) -> Result<roll::Result, String> {
    match solver.solve() {
        Ok(result) => {
            {
                // solver.trim_reason();
                let mut data = ctx.data.write().await;
                let roll_map = data.get_mut::<RollMap>().unwrap_or_log();
                roll_map.insert(msg.author.to_string(), solver);
            }
            Ok(result)
        }
        Err(error) => Err(format_havok_error(error)),
    }
}

pub fn search_critics(result: &roll::Result) -> Result<HashSet<Critic>, serenity::Error> {
    let mut critics = HashSet::new();
    match result.get_result() {
        roll::Kind::Single(result) => {
            search_critic(result, &mut critics)?;
            Ok(critics)
        }
        roll::Kind::Multi(results) => {
            for result in results.iter() {
                search_critic(result, &mut critics)?;
                if critics.len() >= 2 {
                    return Ok(critics);
                }
            }
            Ok(critics)
        }
    }
}

fn search_critic(
    result: &roll::kind::Single,
    critics: &mut HashSet<Critic>,
) -> Result<(), serenity::Error> {
    let mut has_roll = false;
    for kind in result.get_history().iter() {
        match kind {
            History::Roll(results) => {
                has_roll = true;
                for result in results.iter() {
                    match result.critic {
                        Critic::Not => {}
                        _ => {
                            critics.insert(result.critic);
                        }
                    }
                }
            }
            History::Fudge(_) => has_roll = true,
            _ => (),
        }
    }
    if has_roll {
        Ok(())
    } else {
        Err(serenity::Error::Other("no roll found"))
    }
}

pub fn check_critics(critics: Result<HashSet<Critic>, serenity::Error>) -> Option<HashSet<Critic>> {
    match critics {
        Ok(critics) => {
            if !critics.is_empty() {
                Some(critics)
            } else {
                None
            }
        }
        Err(_) => {
            let mut critics = HashSet::new();
            critics.insert(Critic::Not);
            Some(critics)
        }
    }
}
