use pest::iterators::Pair;
use pest::iterators::Pairs;
use pest_derive::Parser;

/// Pest parser
#[derive(Parser)]
#[grammar = "havok.pest"]
pub struct Parser;

impl Parser {
    pub fn extract_dice(expr: &mut Pairs<Rule>) -> Option<String> {
        for inner in expr.by_ref() {
            match inner.as_rule() {
                Rule::expr | Rule::block_expr => {
                    return Self::extract_dice(&mut inner.into_inner())
                }
                Rule::dice => return Some(inner.as_str().trim().to_owned()),
                _ => (),
            };
        }
        None
    }

    pub fn extract_modifier_value(modifier: Pair<Rule>) -> Option<u64> {
        modifier
            .into_inner()
            .next()
            .map(|value| value.as_str().parse::<u64>().unwrap())
    }
}
