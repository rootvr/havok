#![allow(unused)]

/// Keep an integer or floating point constant
#[derive(Debug, Clone)]
pub enum Constant {
    Integer(i64),
    Float(f64),
}

impl Constant {
    pub fn get_value(&self) -> i64 {
        match *self {
            Constant::Integer(n) => n,
            Constant::Float(n) => n as i64, // TODO(resu) FIXME why cast
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Constant::Integer(n) => write!(f, "{n}"),
            Constant::Float(n) => write!(f, "{n}"),
        }
    }
}
