use crate::discord::utils::get_chat_id;

use super::model::{AliasEntry, Chunk};
use super::AliasMap;
use serenity::framework::standard::Args;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tracing_unwrap::OptionExt;

pub async fn parse_alias(
    ctx: &Context,
    msg: &Message,
    args: Args,
) -> Result<(String, bool), String> {
    let data = ctx.data.read().await;
    let all = data.get::<AliasMap>().unwrap_or_log();
    all.expand_alias(args.rest(), get_chat_id(msg), *msg.author.id.as_u64(), true)
}

pub fn split_command(mut command: &str) -> Result<Vec<Chunk>, String> {
    let mut chunks = Vec::new();
    let comment = command.find(':').map(|comment_start| {
        let comment = command[comment_start..].to_string();
        command = &command[..comment_start - 1];
        Chunk::Comment(comment)
    });
    while let Some(alias_start) = command.find('$') {
        let alias_end = command[alias_start..]
            .find(|c: char| c.is_whitespace() || c == '+' || c == '-' || c == '*' || c == '/')
            .or(Some(command.len() - alias_start))
            .unwrap_or_log()
            + alias_start;
        if alias_start > 0 {
            // there some text before the alias, save it
            chunks.push(Chunk::Expr(command[..alias_start].to_string()))
        }
        let alias = split_alias_args(&command[alias_start + 1..alias_end])?;
        chunks.push(Chunk::Alias(alias));
        command = &command[alias_end..];
    }
    if !command.is_empty() {
        chunks.push(Chunk::Expr(command.to_string()));
    }
    if let Some(comment) = comment {
        chunks.push(comment);
    }
    Ok(chunks)
}

fn split_alias_args(command: &str) -> Result<AliasEntry, String> {
    match command.find('|') {
        Some(pipe_index) => {
            if pipe_index == 0 {
                return Err("Syntax error, no parameter before `|`".to_string());
            }
            if pipe_index >= command.len() - 1 {
                return Err("Syntax error, no alias after `|`".to_string());
            }
            let alias_args = &command[..pipe_index];
            let alias_name = &command[pipe_index + 1..];
            if alias_name.find('|').is_some() {
                return Err("Syntax error, can't have another `|`".to_string());
            }
            let alias_args: Vec<String> = alias_args.split(',').map(|s| s.to_owned()).collect();
            Ok(AliasEntry {
                name: alias_name.to_owned(),
                args: alias_args,
            })
        }
        None => Ok(AliasEntry {
            name: command.to_owned(),
            args: Vec::new(),
        }),
    }
}

pub fn collect_expanded(mut expanded: Vec<Chunk>) -> Result<String, String> {
    let mut had_error = false;
    expanded.sort();
    let string = expanded
        .into_iter()
        .fold(String::new(), |acc, part| match part {
            Chunk::Alias(_) | Chunk::Expr(_) | Chunk::Comment(_) => {
                if !had_error {
                    format!("{}{}", acc, part)
                } else {
                    acc
                }
            }
            Chunk::Err(_) => {
                if !had_error {
                    had_error = true;
                    // no errors before, wipe all and only keep errors from now on
                    if !part.is_empty() {
                        format!("{}", part)
                    } else {
                        "".to_string()
                    }
                } else if !part.is_empty() {
                    format!("{}\n{}", acc, part)
                } else {
                    acc
                }
            }
        });
    if had_error {
        Err(string)
    } else {
        Ok(string)
    }
}
