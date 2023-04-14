use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use tracing_unwrap::ResultExt;

const ALIAS_DIR: &str = ".havok";

pub struct AliasContainer(HashMap<u64, AliasData>);

impl AliasContainer {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn list_alias(&self, chat_id: u64, user_id: u64) -> (Vec<String>, Vec<String>) {
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

    pub fn load_alias_data(&mut self, chat_id: u64) -> std::io::Result<&'static str> {
        let mut path = PathBuf::from(ALIAS_DIR);
        if path.exists() {
            path.push(format!("{}.ron", chat_id));
            match fs::read_to_string(path) {
                Ok(content) => {
                    let data: AliasData =
                        ron::de::from_str(&content).unwrap_or_else(|_| AliasData::new());
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
        let data = self.entry(chat_id).or_insert_with(AliasData::new);
        let user_defs = data.user_defs.entry(user_id).or_insert_with(HashMap::new);
        let alias = alias.to_lowercase();
        let msg = format!(
            "**info** *alias* `${}` *set for user* **{}**",
            alias, user_name
        );
        user_defs.insert(alias, command);
        msg
        //     }
        //     Err(s) => s,
        // }
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
pub struct AliasData {
    pub global_defs: HashMap<String, String>,
    pub user_defs: HashMap<u64, HashMap<String, String>>,
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
struct AliasEntry {
    name: String,
    args: Vec<String>,
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

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Chunk {
    Alias(AliasEntry),
    Expr(String),
    Comment(String),
    Err(String),
}

impl Chunk {
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
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
