use super::model::AliasContainer;
use serenity::prelude::TypeMapKey;

pub struct AliasMap;

impl TypeMapKey for AliasMap {
    type Value = AliasContainer;
}
