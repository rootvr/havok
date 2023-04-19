use havok_lib::solver::Solver;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;

pub(crate) struct RollMap;

impl TypeMapKey for RollMap {
    type Value = HashMap<String, Solver>;
}
