use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::Mutex;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

pub struct ShardManagerMap;

impl TypeMapKey for ShardManagerMap {
    type Value = Arc<Mutex<ShardManager>>;
}
