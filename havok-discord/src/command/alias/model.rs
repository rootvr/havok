use super::utils::collect_expanded;
use super::utils::split_command;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use tracing::info;
use tracing_unwrap::ResultExt;

const ALIAS_DIR: &str = ".havok";

pub(crate) struct AliasContainer(HashMap<u64, AliasData>);

impl AliasContainer {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn expand_alias(
        &self,
        command: &str,
        chat_id: u64,
        user_id: u64,
        expand_args: bool,
    ) -> Result<(String, bool), String> {
        let mut alias_seen = HashSet::new();
        let chunks = split_command(command)?;
        let has_alias = chunks.iter().any(|e| matches!(e, Chunk::Alias(_)));
        Ok((
            collect_expanded(self.user_alias_expansion(
                chunks,
                chat_id,
                user_id,
                &mut alias_seen,
                expand_args,
            )?)?,
            has_alias,
        ))
    }

    fn user_alias_expansion(
        &self,
        chunks: Vec<Chunk>,
        chat_id: u64,
        user_id: u64,
        alias_seen: &mut HashSet<String>,
        expand_args: bool,
    ) -> Result<Vec<Chunk>, String> {
        chunks
            .into_iter()
            .try_fold(Vec::new(), |mut acc, chunk| -> Result<Vec<Chunk>, String> {
                match chunk {
                    chunk @ Chunk::Expr(_) | chunk @ Chunk::Err(_) | chunk @ Chunk::Comment(_) => {
                        acc.push(chunk)
                    }
                    Chunk::Alias(alias) => {
                        if alias_seen.contains(&alias.name) {
                            acc.push(Chunk::Err(format!(
                                "**error** `${}` *already expanded (cycle definition)*",
                                alias
                            )));
                        } else {
                            alias_seen.insert(alias.name.clone());
                            match self.get_alias_value(&alias.name, chat_id, user_id) {
                                Ok(Some(expanded)) => {
                                    let expanded = if expand_args {
                                        self.apply_args(&alias.args, expanded)?
                                    } else {
                                        expanded
                                    };
                                    let expanded = split_command(&expanded)?;
                                    let mut expanded = self.user_alias_expansion(
                                        expanded,
                                        chat_id,
                                        user_id,
                                        alias_seen,
                                        expand_args,
                                    )?;
                                    acc.append(&mut expanded);
                                }
                                Ok(None) => self.get_global_value_and_expand(
                                    &alias,
                                    chat_id,
                                    &mut acc,
                                    alias_seen,
                                    expand_args,
                                )?,
                                Err(error) => acc.push(Chunk::Err(error)),
                            }
                        }
                    }
                };
                Ok(acc)
            })
    }

    fn apply_args(&self, args: &[String], expansion: String) -> Result<String, String> {
        let mut applied = String::new();
        let mut slice = expansion.as_str();
        while let Some(pos_start) = slice.find('%') {
            let pos_end = {
                let index_end = slice[pos_start + 1..].find(|c: char| !c.is_ascii_digit());
                match index_end {
                    Some(index_end) => index_end + pos_start + 1,
                    None => slice.len(),
                }
            };
            let index = {
                let pos = slice[pos_start + 1..pos_end]
                    .parse::<usize>()
                    .map_err(|e| format!("Can't parse: {}", e))?;
                pos - 1
            };
            if index >= args.len() {
                return Err("Parameter reference is above number of parameter".to_string());
            }
            if pos_start > 0 {
                applied.push_str(&slice[..pos_start]);
            }
            applied.push_str(args.get(index).unwrap().as_str());
            slice = &slice[pos_end..];
        }
        applied.push_str(slice);
        Ok(applied)
    }

    fn expand_global_alias(
        &self,
        cmd: &str,
        chat_id: u64,
        expand_args: bool,
    ) -> Result<(String, bool), String> {
        let mut alias_seen = HashSet::new();
        let chunks = split_command(cmd)?;
        let has_alias = chunks.iter().any(|e| matches!(e, Chunk::Alias(_)));
        Ok((
            collect_expanded(self.global_alias_expansion(
                chunks,
                chat_id,
                &mut alias_seen,
                expand_args,
            )?)?,
            has_alias,
        ))
    }

    fn get_alias_value(
        &self,
        alias: &str,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Option<String>, String> {
        match self.get(&chat_id) {
            Some(data) => match data.user_defs.get(&user_id) {
                Some(user_defs) => match user_defs.get(alias) {
                    p @ Some(_) => Ok(p.cloned()),
                    None => Ok(None),
                },
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn get_global_value_and_expand(
        &self,
        alias: &AliasEntry,
        chat_id: u64,
        acc: &mut Vec<Chunk>,
        alias_seen: &mut HashSet<String>,
        expand_args: bool,
    ) -> Result<(), String> {
        match self.get_global_alias_value(&alias.name, chat_id) {
            Ok(Some(expanded)) => {
                let expanded = if expand_args {
                    self.apply_args(&alias.args, expanded)?
                } else {
                    expanded
                };
                let expanded = split_command(&expanded)?;
                let mut expanded =
                    self.global_alias_expansion(expanded, chat_id, alias_seen, expand_args)?;
                acc.append(&mut expanded);
            }
            Ok(None) => {
                acc.push(Chunk::Err(format!(
                    "**error** `${}` *not found in* **global** *aliases*",
                    alias
                )));
            }
            Err(error) => acc.push(Chunk::Err(error)),
        }
        Ok(())
    }

    fn global_alias_expansion(
        &self,
        chunks: Vec<Chunk>,
        chat_id: u64,
        alias_seen: &mut HashSet<String>,
        expand_args: bool,
    ) -> Result<Vec<Chunk>, String> {
        chunks.into_iter().try_fold(Vec::new(), |mut acc, part| {
            match part {
                chunk @ Chunk::Expr(_) | chunk @ Chunk::Err(_) | chunk @ Chunk::Comment(_) => {
                    acc.push(chunk)
                }
                Chunk::Alias(alias) => {
                    if alias_seen.contains(&alias.name) {
                        acc.push(Chunk::Err(format!(
                            "**error** `{}` *already expanded (cycle definition)*",
                            alias.name
                        )));
                    } else if alias.name.chars().all(|c| c.is_lowercase()) {
                        // reference to a future user alias
                        acc.push(Chunk::Expr(format!("${}", alias)))
                    } else {
                        alias_seen.insert(alias.name.clone());
                        self.get_global_value_and_expand(
                            &alias,
                            chat_id,
                            &mut acc,
                            alias_seen,
                            expand_args,
                        )?;
                    }
                }
            };
            Ok(acc)
        })
    }

    fn get_global_alias_value(&self, alias: &str, chat_id: u64) -> Result<Option<String>, String> {
        match self.get(&chat_id) {
            Some(data) => match data.global_defs.get(&alias.to_uppercase()) {
                p @ Some(_) => Ok(p.cloned()),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub(crate) fn list_alias(&self, chat_id: u64, user_id: u64) -> (Vec<String>, Vec<String>) {
        match self.get(&chat_id) {
            Some(data) => (
                match data.user_defs.get(&user_id) {
                    Some(user_aliases) => user_aliases
                        .iter()
                        .map(|(k, v)| format!("***{}*** = `{}`", k, v))
                        .collect(),
                    None => vec![],
                },
                data.global_defs
                    .iter()
                    .map(|(k, v)| format!("***{}*** = `{}`", k, v))
                    .collect(),
            ),
            None => (vec![], vec![]),
        }
    }

    pub(crate) fn load_alias_data(&mut self, chat_id: u64) -> std::io::Result<&'static str> {
        let mut path = PathBuf::from(ALIAS_DIR);
        if path.exists() {
            path.push(format!("{}.ron", chat_id));
            match fs::read_to_string(path) {
                Ok(content) => {
                    let data: AliasData =
                        ron::de::from_str(&content).unwrap_or_else(|_| AliasData::new());
                    self.insert(chat_id, data);
                }
                Err(error) => return Err(error),
            }
        }
        Ok("**info** *config loaded*")
    }

    pub(crate) fn save_alias_data(&self, chat_id: u64) -> std::io::Result<&'static str> {
        let send = match self.get(&chat_id) {
            Some(data) => {
                let ser = ron::ser::to_string_pretty(&data, Default::default()).unwrap_or_log();
                let mut path = PathBuf::from(ALIAS_DIR);
                fs::create_dir_all(ALIAS_DIR).expect_or_log("unable to create dir `.havok`");
                path.push(format!("{}.ron", chat_id));
                fs::write(path, ser.as_bytes())?;
                "**info** *config saved*"
            }
            None => "**error** *nothing to save*",
        };
        Ok(send)
    }

    pub(crate) fn set_user_alias(
        &mut self,
        alias: String,
        command: String,
        chat_id: u64,
        user_id: u64,
        user_name: &str,
    ) -> String {
        let alias = alias.trim_matches(|c: char| c == '$' || c.is_whitespace());
        match self.expand_alias(&command, chat_id, user_id, false) {
            Ok(_) => {
                let data = self.entry(chat_id).or_insert_with(AliasData::new);
                let user_defs = data.user_defs.entry(user_id).or_insert_with(HashMap::new);
                let alias = alias.to_lowercase();
                let send = format!(
                    "**info** *alias* `${}` *set for user* **{}**",
                    alias, user_name
                );
                user_defs.insert(alias, command);
                send
            }
            Err(error) => error,
        }
    }

    pub(crate) fn del_user_alias(&mut self, alias: &str, chat_id: u64, user_id: u64) -> String {
        let alias = alias
            .trim_matches(|c: char| c == '$' || c.is_whitespace())
            .to_lowercase();
        let data = self.entry(chat_id).or_insert_with(AliasData::new);
        match data.user_defs.get_mut(&user_id) {
            Some(user_defs) => match user_defs.remove(&alias) {
                Some(_) => format!("**info** *alias* `${}` *deleted*", alias),
                None => "**error** *alias to delete not found*".to_string(),
            },
            None => "**error** *alias to delete not found*".to_string(),
        }
    }

    pub(crate) fn clear_user_aliases(&mut self, chat_id: u64, user_id: u64) -> &'static str {
        let data = self.entry(chat_id).or_insert_with(AliasData::new);
        match data.user_defs.get_mut(&user_id) {
            Some(user_defs) => {
                user_defs.clear();
                "**info** *all your aliases have been deleted*"
            }
            None => "**info** *you don't have any alias set*",
        }
    }

    pub(crate) fn set_global_alias(
        &mut self,
        alias: String,
        command: String,
        chat_id: u64,
    ) -> String {
        let alias = alias.trim_matches(|c: char| c == '$' || c.is_whitespace());
        match self.expand_global_alias(&command, chat_id, false) {
            Ok(_) => {
                let alias = alias.to_uppercase();
                let data = self.entry(chat_id).or_insert_with(AliasData::new);
                let send = format!("**info** *global alias* `${}` *set*", alias);
                data.global_defs.insert(alias, command);
                send
            }
            Err(error) => error,
        }
    }

    pub(crate) fn del_global_alias(&mut self, alias: &str, chat_id: u64) -> String {
        let alias = alias
            .trim_matches(|c: char| c == '$' || c.is_whitespace())
            .to_uppercase();
        let data = self.entry(chat_id).or_insert_with(AliasData::new);
        data.global_defs.remove(&alias);
        format!("**info** *global alias* `${}` *deleted*", alias)
    }

    pub(crate) fn clear_aliases(&mut self, chat_id: u64) -> &'static str {
        if let Some(data) = self.get_mut(&chat_id) {
            data.global_defs.clear();
        }
        "**info** *aliases cleared*"
    }

    pub(crate) fn save_all(&self) {
        let keys: Vec<_> = { self.keys().cloned().collect() };
        for chat_id in keys {
            if let Some(data) = self.get(&chat_id) {
                let ser = ron::ser::to_string_pretty(&data, Default::default()).unwrap();
                let mut path = PathBuf::from(ALIAS_DIR);
                fs::create_dir_all(ALIAS_DIR).expect_or_log("unable to create dir `.havok`");
                path.push(format!("{}.ron", chat_id));
                std::fs::write(&path, ser.as_bytes()).expect_or_log("unable to dump aliases");
                info!("Aliases dumped to `{:?}`", path);
            }
        }
    }
}

impl Deref for AliasContainer {
    type Target = HashMap<u64, AliasData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AliasContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AliasData {
    pub(crate) global_defs: HashMap<String, String>,
    pub(crate) user_defs: HashMap<u64, HashMap<String, String>>,
}

impl AliasData {
    fn new() -> Self {
        Self {
            global_defs: HashMap::new(),
            user_defs: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AliasEntry {
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
}

impl std::fmt::Display for AliasEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}|{}", self.args.iter().format(", "), self.name)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Chunk {
    Alias(AliasEntry),
    Expr(String),
    Comment(String),
    Err(String),
}

impl Chunk {
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Chunk::Alias(a) => a.name.is_empty(),
            Chunk::Expr(e) => e.is_empty(),
            Chunk::Err(e) => e.is_empty(),
            Chunk::Comment(c) => c.is_empty(),
        }
    }
}

impl Ord for Chunk {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Chunk::Alias(_), Chunk::Comment(_)) // less
            | (Chunk::Expr(_), Chunk::Comment(_)) => Ordering::Less,
            (Chunk::Alias(_), Chunk::Alias(_)) // equal
            | (Chunk::Alias(_), Chunk::Expr(_))
            | (Chunk::Expr(_), Chunk::Alias(_))
            | (Chunk::Comment(_), Chunk::Comment(_))
            | (Chunk::Expr(_), Chunk::Expr(_)) => Ordering::Equal,
            (Chunk::Comment(_), Chunk::Alias(_)) // greater
            | (Chunk::Comment(_), Chunk::Expr(_))
            | (Chunk::Comment(_), Chunk::Err(_))
            | (Chunk::Err(_), Chunk::Alias(_))
            | (Chunk::Err(_), Chunk::Expr(_))
            | (Chunk::Err(_), Chunk::Comment(_))
            | (Chunk::Err(_), Chunk::Err(_))
            | (Chunk::Alias(_), Chunk::Err(_))
            | (Chunk::Expr(_), Chunk::Err(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for Chunk {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chunk::Alias(a) => write!(f, "{}", &a),
            Chunk::Expr(e) => write!(f, "{}", &e),
            Chunk::Comment(c) => write!(f, "{}", &c),
            Chunk::Err(e) => write!(f, "{}", &e),
        }
    }
}
