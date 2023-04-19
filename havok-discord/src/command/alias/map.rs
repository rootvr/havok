use super::model::AliasContainer;
use serenity::prelude::TypeMapKey;

pub(crate) struct AliasMap;

impl TypeMapKey for AliasMap {
    type Value = AliasContainer;
}
