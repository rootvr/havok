use crate::dice;
use crate::error::Result;
use crate::evaluator::Evaluator;
use crate::parser;
use crate::roll;
use pest::iterators::Pair;
use pest::Parser;
use rand::CryptoRng;
use rand::Rng;

/// Default random dice roller
pub(crate) struct RandomSource<'a, T: Rng + CryptoRng> {
    pub(crate) generator: &'a mut T,
}

impl<T: Rng + CryptoRng> roll::Source for RandomSource<'_, T> {
    fn throw(&mut self, sides: u64) -> u64 {
        self.generator.gen_range(1..1 + sides)
    }
}

const REASON_SIGIL: char = ':';

/// Represent a solver and holds the query string
#[derive(Clone, Debug)]
pub struct Solver(String);

impl Solver {
    pub fn new(input: &str) -> Result<Self> {
        Ok(Solver(input.to_owned()))
    }

    /// Solve the roll expression using the default Rng source
    pub fn solve(&self) -> Result<roll::Result> {
        self.solve_with(&mut rand::thread_rng())
    }

    /// Solve the roll expression using the provided Rng source
    pub fn solve_with<S: Rng + CryptoRng>(&self, generator: &mut S) -> Result<roll::Result> {
        self.solve_with_source(&mut RandomSource { generator })
    }

    /// Solve the roll expression using the provided source
    pub fn solve_with_source<S: roll::Source>(&self, source: &mut S) -> Result<roll::Result> {
        let mut pairs = parser::Parser::parse(parser::Rule::command, &self.0)?;
        let expr = pairs.next().unwrap();
        let mut result = match expr.as_rule() {
            parser::Rule::expr => {
                roll::Result::new_single(Evaluator::eval(expr.into_inner(), source, false)?)
            }
            parser::Rule::repeated_expr => Solver::solve_multi(expr, source)?,
            _ => unreachable!(),
        };
        if let Some(reason) = pairs.next() {
            if reason.as_rule() == parser::Rule::reason {
                result.add_reason(reason.as_str()[1..].trim().to_owned());
            }
        }
        Ok(result)
    }

    /// Solve a multi roll expression using the provided source
    fn solve_multi<S: roll::Source>(
        pairs: Pair<parser::Rule>,
        source: &mut S,
    ) -> Result<roll::Result> {
        let mut pairs = pairs.into_inner();
        let expr = pairs.next().unwrap();
        let repeat = pairs.next().unwrap();
        let (iters, sum, sort) = match repeat.as_rule() {
            parser::Rule::nb_dice => (repeat.as_str().parse::<u64>().unwrap(), false, false),
            parser::Rule::add => (
                pairs.next().unwrap().as_str().parse::<u64>().unwrap(),
                true,
                false,
            ),
            parser::Rule::sort => (
                pairs.next().unwrap().as_str().parse::<u64>().unwrap(),
                false,
                true,
            ),
            _ => unreachable!(),
        };
        let results: Result<Vec<roll::kind::Single>> =
            (0..iters).try_fold(Vec::new(), |mut res, _| {
                let c = Evaluator::eval(expr.clone().into_inner(), source, false)?;
                res.push(c);
                Ok(res)
            });
        let mut results = results?;
        if sort {
            results.sort_unstable_by(|a, b| a.get_total().partial_cmp(&b.get_total()).unwrap());
        }
        let total = if sum {
            Some(results.iter().fold(0, |acc, curr| acc + curr.get_total()))
        } else {
            None
        };
        Ok(roll::Result::new_multi(results, total))
    }

    /// Return an iterator on the dices in the roll expression
    pub fn dices(&self) -> Result<dice::Iter> {
        let inner = parser::Parser::parse(parser::Rule::command, &self.0)?
            .next()
            .unwrap()
            .into_inner();
        Ok(dice::Iter { inner })
    }

    /// Return the query string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Removes the reason from the query
    pub fn trim_reason(&mut self) {
        if let Some(index) = self.0.find(REASON_SIGIL) {
            self.0 = self.0[..index].to_owned()
        }
    }
}
