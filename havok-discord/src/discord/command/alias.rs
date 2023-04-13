use crate::discord::send_msg;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::prelude::TypeMapKey;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use tracing::info;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

const ALIAS_DIR: &str = ".havok";

fn get_chat_id(msg: &Message) -> u64 {
    match msg.guild_id {
        Some(id) => *id.as_u64(),
        None => *msg.channel_id.as_u64(),
    }
}

async fn get_user_name(ctx: &Context, msg: &Message) -> String {
    match msg.guild_id {
        Some(id) => msg
            .author
            .nick_in(ctx, id)
            .await
            .unwrap_or_else(|| msg.author.name.to_owned()),
        None => msg.author.name.to_owned(),
    }
}

/// channel id, Data
pub struct All(HashMap<u64, Data>);

impl All {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn list_alias(&self, chat_id: u64, user_id: u64) -> (Vec<String>, Vec<String>) {
        match self.get(&chat_id) {
            Some(data) => (
                match data.user_aliases.get(&user_id) {
                    Some(user_aliases) => user_aliases
                        .iter()
                        .map(|(k, v)| format!("***{}*** = `{}`", k, v))
                        .collect(),
                    None => vec![],
                },
                data.global_aliases
                    .iter()
                    .map(|(k, v)| format!("***{}*** = `{}`", k, v))
                    .collect(),
            ),
            None => (vec![], vec![]),
        }
    }

    pub fn load_alias_data(&mut self, chat_id: u64) -> std::io::Result<&'static str> {
        let mut path = PathBuf::from(ALIAS_DIR);
        if path.exists() {
            path.push(format!("{}.ron", chat_id));
            match fs::read_to_string(path) {
                Ok(content) => {
                    let data: Data = ron::de::from_str(&content).unwrap_or_else(|_| Data::new());
                    self.insert(chat_id, data);
                }
                Err(e) => return Err(e),
            }
        }
        Ok("**info** *config loaded*")
    }

    pub fn save_alias_data(&self, chat_id: u64) -> std::io::Result<&'static str> {
        let msg = match self.get(&chat_id) {
            Some(data) => {
                let ser = ron::ser::to_string_pretty(&data, Default::default()).unwrap_or_log();
                let mut path = PathBuf::from(ALIAS_DIR);
                fs::create_dir_all(ALIAS_DIR).expect_or_log("unable to create dir `.havok`");
                path.push(format!("{}.ron", chat_id));
                fs::write(path, ser.as_bytes())?;
                "**info** *config saved"
            }
            None => "**error** *nothing to save*",
        };
        Ok(msg)
    }

    pub fn set_user_alias(
        &mut self,
        alias: String,
        command: String,
        chat_id: u64,
        user_id: u64,
        user_name: &str,
    ) -> String {
        let alias = alias.trim_matches(|c: char| c == '$' || c.is_whitespace());
        // TODO(resu): expand alias recursively
        // match self.expand_alias(&command, chat_id, user_id, false) {
        //     Ok(_) => {
        let data = self.entry(chat_id).or_insert_with(Data::new);
        let user_aliases = data
            .user_aliases
            .entry(user_id)
            .or_insert_with(HashMap::new);
        let alias = alias.to_lowercase();
        let msg = format!(
            "**info** *alias* `${}` *set for user* **{}**",
            alias, user_name
        );
        user_aliases.insert(alias, command);
        msg
        //     }
        //     Err(s) => s,
        // }
    }
}

impl Deref for All {
    type Target = HashMap<u64, Data>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for All {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    /// alias, command
    pub global_aliases: HashMap<String, String>,
    /// user id, map of aliases (alias, command)
    pub user_aliases: HashMap<u64, HashMap<String, String>>,
}

impl Data {
    fn new() -> Self {
        Self {
            global_aliases: HashMap::new(),
            user_aliases: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AliasData {
    name: String,
    args: Vec<String>,
}

impl std::fmt::Display for AliasData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}|{}", self.args.iter().format(", "), self.name)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SplitPart {
    Alias(AliasData),
    Expr(String),
    Comment(String),
    Err(String),
}

impl SplitPart {
    pub fn is_empty(&self) -> bool {
        match self {
            SplitPart::Alias(a) => a.name.is_empty(),
            SplitPart::Expr(e) => e.is_empty(),
            SplitPart::Err(e) => e.is_empty(),
            SplitPart::Comment(c) => c.is_empty(),
        }
    }
}

impl Ord for SplitPart {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (SplitPart::Alias(_), SplitPart::Comment(_))
            | (SplitPart::Expr(_), SplitPart::Comment(_)) => Ordering::Less,
            (SplitPart::Alias(_), SplitPart::Alias(_))
            | (SplitPart::Alias(_), SplitPart::Expr(_))
            | (SplitPart::Expr(_), SplitPart::Alias(_))
            | (SplitPart::Comment(_), SplitPart::Comment(_))
            | (SplitPart::Expr(_), SplitPart::Expr(_)) => Ordering::Equal,
            (SplitPart::Comment(_), SplitPart::Alias(_))
            | (SplitPart::Comment(_), SplitPart::Expr(_))
            | (SplitPart::Comment(_), SplitPart::Err(_))
            | (SplitPart::Err(_), SplitPart::Alias(_))
            | (SplitPart::Err(_), SplitPart::Expr(_))
            | (SplitPart::Err(_), SplitPart::Comment(_))
            | (SplitPart::Err(_), SplitPart::Err(_))
            | (SplitPart::Alias(_), SplitPart::Err(_))
            | (SplitPart::Expr(_), SplitPart::Err(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for SplitPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for SplitPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplitPart::Alias(a) => write!(f, "{}", &a),
            SplitPart::Expr(e) => write!(f, "{}", &e),
            SplitPart::Comment(c) => write!(f, "{}", &c),
            SplitPart::Err(e) => write!(f, "{}", &e),
        }
    }
}

pub struct AliasContainer;

impl TypeMapKey for AliasContainer {
    type Value = All;
}

#[group]
#[prefix = "alias"]
#[description = "alias management group"]
#[commands(list_alias, set_user_alias, save_alias, load_alias)]
struct Alias;

#[command]
#[aliases("ls", "list")]
#[min_args(0)]
async fn list_alias(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Received `{:?}`", msg.content);
    let send = {
        let data = ctx.data.read().await;
        let all = data.get::<AliasContainer>().unwrap_or_log();
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
    send_msg(ctx, msg, &send).await?;
    Ok(())
}

#[command]
#[aliases("su", "set")]
#[min_args(2)]
async fn set_user_alias(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    info!("Received `{:?}`", msg.content);
    let send = {
        let alias = args.single::<String>().unwrap_or_log();
        let command = args.rest().to_string();
        let mut data = ctx.data.write().await;
        let all = data.get_mut::<AliasContainer>().unwrap_or_log();
        all.set_user_alias(
            alias,
            command,
            get_chat_id(msg),
            *msg.author.id.as_u64(),
            &get_user_name(ctx, msg).await,
        )
    };
    send_msg(ctx, msg, &send).await?;
    Ok(())
}

#[command]
#[aliases("sv", "save")]
#[max_args(0)]
async fn save_alias(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    info!("Received `{:?}`", msg.content);
    let send = {
        let data = ctx.data.read().await;
        let all = data.get::<AliasContainer>().unwrap_or_log();
        all.save_alias_data(get_chat_id(msg))?
    };
    send_msg(ctx, msg, send).await?;
    Ok(())
}

#[command]
#[aliases("ld", "load")]
#[max_args(0)]
async fn load_alias(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    info!("Received `{:?}`", msg.content);
    let send = {
        let mut data = ctx.data.write().await;
        let all = data.get_mut::<AliasContainer>().unwrap_or_log();
        all.load_alias_data(get_chat_id(msg))?
    };
    send_msg(ctx, msg, send).await?;
    Ok(())
}
