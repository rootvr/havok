use crate::constant;
use crate::dice;
use itertools::Itertools;

/// Keep a single step of the history that led to the final result
#[derive(Debug, Clone)]
pub enum History {
    OpenParen,
    CloseParen,
    Fudge(Vec<u64>),
    Operator(&'static str),
    Roll(Vec<dice::Result>),
    Constant(constant::Constant),
}

impl std::fmt::Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            History::OpenParen => write!(f, "(")?,
            History::CloseParen => write!(f, ")")?,
            History::Fudge(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|r| match r {
                        r if *r <= 2 => "-",
                        r if *r <= 4 => "▢",
                        _ => "+",
                    })
                    .format(", ")
            )?,
            History::Operator(o) => write!(f, "{o}")?,
            History::Roll(v) => write!(
                f,
                "[{}]",
                v.iter().map(|r| r.value.to_string()).format(", ")
            )?,
            History::Constant(v) => write!(f, "{v}")?,
        }
        Ok(())
    }
}
