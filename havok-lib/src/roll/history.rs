use crate::constant;
use crate::dice;
use itertools::Itertools;

/// Keep a single step of the history that led to the final result
#[derive(Debug, Clone)]
pub enum History {
    Roll(Vec<dice::Result>),
    Fudge(Vec<u64>),
    Constant(constant::Constant),
    Operator(&'static str),
    OpenParen,
    CloseParen,
}

impl std::fmt::Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            History::Roll(v) => write!(
                f,
                "[{}]",
                v.iter().map(|r| r.value.to_string()).format(", ")
            )?,
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
            History::Constant(v) => write!(f, "{}", v.to_string())?,
            History::Operator(o) => write!(f, "{o}")?,
            History::OpenParen => write!(f, "(")?,
            History::CloseParen => write!(f, ")")?,
        }
        Ok(())
    }
}
