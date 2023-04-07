use crate::dice;

/// Keep a collection of dice results with modifier
pub struct Result {
    pub modifier: super::Modifier,
    pub results: Vec<dice::Result>,
}
