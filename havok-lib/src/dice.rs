pub(crate) mod modifier;

use crate::parser;
use pest::iterators::Pairs;
use std::ops::Deref;

/// Mark if a dice result is a critic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Critic {
    Max,
    Min,
    Not,
}

/// Keep one dice result with critic marker
#[derive(Debug, Clone, Copy)]
pub struct Result {
    pub value: u64,
    pub critic: Critic,
}

impl Result {
    pub fn new(value: u64, sides: u64) -> Self {
        Result {
            value,
            critic: match value {
                v if v == sides => Critic::Max,
                v if v == 1 => Critic::Min,
                _ => Critic::Not,
            },
        }
    }
}

impl PartialEq for Result {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Result {}

impl PartialOrd for Result {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Result {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Deref for Result {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Iterator that lazily returns each dice in the query
pub struct Iter<'a> {
    pub inner: Pairs<'a, parser::Rule>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        parser::Parser::extract_dice(&mut self.inner)
    }
}

/// Optional dice modifier with the amount of dices to keep or drop
#[derive(Clone, PartialEq)]
pub enum Modifier {
    Fudge,
    KeepLow(usize),
    DropLow(usize),
    KeepHigh(usize),
    DropHigh(usize),
    None(parser::Rule),
    TargetEnum(Vec<u64>),
    TargetDoubleFailure(u64, u64, u64),
}
