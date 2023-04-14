use crate::parser;
use once_cell::sync::Lazy;
use pest::iterators::Pair;
use pest::pratt_parser::PrattParser;

/// Singleton wrapper for a Pratt parser
pub struct Climber {
    inner: PrattParser<parser::Rule>,
}

impl Climber {
    pub fn raise<'i, P, F, G, T>(&self, pairs: P, primary: F, infix: G) -> T
    where
        P: Iterator<Item = Pair<'i, parser::Rule>>,
        F: FnMut(Pair<'i, parser::Rule>) -> T,
        G: FnMut(T, Pair<'i, parser::Rule>, T) -> T + 'i,
    {
        self.inner
            .map_primary(primary)
            .map_infix(infix)
            .parse(pairs)
    }
}

pub static C: Lazy<Climber> = Lazy::new(|| {
    use pest::pratt_parser::Assoc;
    use pest::pratt_parser::Op;
    Climber {
        inner: PrattParser::new()
            .op(Op::infix(parser::Rule::add, Assoc::Left))
            .op(Op::infix(parser::Rule::sub, Assoc::Left))
            .op(Op::infix(parser::Rule::mul, Assoc::Left))
            .op(Op::infix(parser::Rule::div, Assoc::Left)),
    }
});
