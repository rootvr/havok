use crate::climber::C;
use crate::dice;
use crate::error::Result;
use crate::parser::Parser;
use crate::parser::Rule;
use crate::roll;
use crate::roll::kind;
use pest::iterators::Pair;
use pest::iterators::Pairs;

mod limits {
    /// Arbitrary limits to avoid oom
    pub(crate) const MAX_DICE_AMOUNT: u64 = 5000;
    pub(crate) const MAX_DICE_SIDES: u64 = 5000;
}

/// Represent an evaluator
pub(crate) struct Evaluator;

impl Evaluator {
    fn eval_explode<S: roll::Source>(
        single: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        pair: Pair<Rule>,
        prior: &dice::Modifier,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(pair).unwrap_or(sides);
        let amount = results.iter().filter(|x| x.value >= value).count() as u64;
        if prior != &dice::Modifier::None(Rule::explode)
            && prior != &dice::Modifier::None(Rule::i_explode)
        {
            single.add_history(results.clone(), false);
        }
        let result = if amount > 0 {
            let result = Self::roll(amount, sides, source);
            single.add_history(result.clone(), false);
            result
        } else {
            results
        };
        (dice::Modifier::None(Rule::explode), result)
    }

    fn eval_indef_explode<S: roll::Source>(
        single: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        pair: Pair<Rule>,
        prior: &dice::Modifier,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(pair).unwrap_or(sides);
        if prior != &dice::Modifier::None(Rule::explode)
            && prior != &dice::Modifier::None(Rule::i_explode)
        {
            single.add_history(results.clone(), false);
        }
        let mut amount = results.into_iter().filter(|x| x.value >= value).count() as u64;
        let mut results = Vec::new();
        while amount > 0 {
            results = Self::roll(amount, sides, source);
            amount = results.iter().filter(|x| x.value >= value).count() as u64;
            single.add_history(results.clone(), false);
        }
        (dice::Modifier::None(Rule::i_explode), results)
    }

    fn eval_reroll<S: roll::Source>(
        single: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        pair: Pair<Rule>,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(pair).unwrap();
        let mut has_rerolled = false;
        let results: Vec<dice::Result> = results
            .into_iter()
            .map(|x| {
                if x.value <= value {
                    has_rerolled = true;
                    Self::roll(1, sides, source)[0]
                } else {
                    x
                }
            })
            .collect();
        if has_rerolled {
            single.add_history(results.clone(), false);
        }
        (dice::Modifier::None(Rule::reroll), results)
    }

    fn eval_indef_reroll<S: roll::Source>(
        single: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        pair: Pair<Rule>,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(pair).unwrap();
        let mut has_rerolled = false;
        let result: Vec<dice::Result> = results
            .into_iter()
            .map(|x| {
                let mut x = x;
                while x.value <= value {
                    has_rerolled = true;
                    x = Self::roll(1, sides, source)[0]
                }
                x
            })
            .collect();
        if has_rerolled {
            single.add_history(result.clone(), false);
        }
        (dice::Modifier::None(Rule::i_reroll), result)
    }

    fn eval_modifier<S: roll::Source>(
        single: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        pair: Pair<Rule>,
        source: &mut S,
        prev: &dice::Modifier,
    ) -> Result<dice::modifier::Result> {
        let (modifier, mut results) = match &pair.as_rule() {
            Rule::explode => Self::eval_explode(single, sides, results, pair, prev, source),
            Rule::i_explode => Self::eval_indef_explode(single, sides, results, pair, prev, source),
            Rule::reroll => Self::eval_reroll(single, sides, results, pair, source),
            Rule::i_reroll => Self::eval_indef_reroll(single, sides, results, pair, source),
            Rule::keep_hi => {
                let value = Parser::extract_modifier_value(pair).unwrap();
                if single.get_history().is_empty() {
                    single.add_history(results.clone(), false);
                }
                (dice::Modifier::KeepHigh(value as usize), results)
            }
            Rule::keep_lo => {
                let value = Parser::extract_modifier_value(pair).unwrap();
                if single.get_history().is_empty() {
                    single.add_history(results.clone(), false);
                }
                (dice::Modifier::KeepLow(value as usize), results)
            }
            Rule::drop_hi => {
                let value = Parser::extract_modifier_value(pair).unwrap();
                if single.get_history().is_empty() {
                    single.add_history(results.clone(), false);
                }
                (dice::Modifier::DropHigh(value as usize), results)
            }
            Rule::drop_lo => {
                let value = Parser::extract_modifier_value(pair).unwrap();
                if single.get_history().is_empty() {
                    single.add_history(results.clone(), false);
                }
                (dice::Modifier::DropLow(value as usize), results)
            }
            Rule::target => {
                let target = pair.into_inner().next().unwrap();
                match target.as_rule() {
                    Rule::number => (
                        dice::Modifier::TargetDoubleFailure(
                            target.as_str().parse::<u64>().unwrap(),
                            0,
                            0,
                        ),
                        results,
                    ),
                    Rule::target_enum => {
                        let values = target.into_inner();
                        let values: Vec<_> =
                            values.map(|p| p.as_str().parse::<u64>().unwrap()).collect();
                        (dice::Modifier::TargetEnum(values), results)
                    }
                    _ => unreachable!(),
                }
            }
            Rule::double_target => {
                let value = Parser::extract_modifier_value(pair).unwrap();
                (dice::Modifier::TargetDoubleFailure(0, 0, value), results)
            }
            Rule::failure => {
                let value = Parser::extract_modifier_value(pair).unwrap();
                (dice::Modifier::TargetDoubleFailure(0, value, 0), results)
            }
            _ => unreachable!("{:#?}", pair),
        };
        let number = match modifier {
            dice::Modifier::KeepHigh(n) | dice::Modifier::KeepLow(n) => {
                if n > results.len() {
                    results.len()
                } else {
                    n
                }
            }
            dice::Modifier::DropHigh(n) | dice::Modifier::DropLow(n) => {
                if n > results.len() {
                    0
                } else {
                    n
                }
            }
            dice::Modifier::None(_)
            | dice::Modifier::TargetDoubleFailure(_, _, _)
            | dice::Modifier::TargetEnum(_)
            | dice::Modifier::Fudge => 0,
        };
        results.sort_unstable();
        let results = match modifier {
            dice::Modifier::KeepHigh(_) => results[results.len() - number..].to_vec(),
            dice::Modifier::KeepLow(_) => results[..number].to_vec(),
            dice::Modifier::DropHigh(_) => results[..results.len() - number].to_vec(),
            dice::Modifier::DropLow(_) => results[number..].to_vec(),
            dice::Modifier::None(_)
            | dice::Modifier::TargetDoubleFailure(_, _, _)
            | dice::Modifier::TargetEnum(_)
            | dice::Modifier::Fudge => results,
        };
        Ok(dice::modifier::Result { results, modifier })
    }

    fn eval_roll<S: roll::Source>(mut dice: Pairs<Rule>, source: &mut S) -> Result<kind::Single> {
        let mut single = kind::Single::new();
        let maybe_amount = dice.next().unwrap();
        let amount = match maybe_amount.as_rule() {
            Rule::nb_dice => {
                dice.next(); // skip `d` token
                let amount = maybe_amount.as_str().parse::<u64>().unwrap();
                if amount > limits::MAX_DICE_AMOUNT {
                    return Err(format!(
                        "exceeded max allowed amount of dices `{}`",
                        limits::MAX_DICE_AMOUNT
                    )
                    .into());
                }
                amount
            }
            Rule::roll => 1,
            _ => unreachable!("{:?}", maybe_amount),
        };
        let pair = dice.next().unwrap();
        let (sides, is_fudge) = match pair.as_rule() {
            Rule::nb_dice => (pair.as_str().parse::<u64>().unwrap(), false),
            Rule::fudge => (6, true),
            _ => unreachable!("{:?}", pair),
        };
        if sides > limits::MAX_DICE_SIDES {
            return Err(format!(
                "exceeded max allowed number of dice sides `{}`",
                limits::MAX_DICE_SIDES
            )
            .into());
        }
        let mut results = Self::roll(amount, sides, source);
        let mut modifier = dice::Modifier::None(Rule::expr);
        let mut maybe_modifier = dice.next();
        if !is_fudge {
            if maybe_modifier.is_some() {
                while maybe_modifier.is_some() {
                    let pair = maybe_modifier.unwrap();
                    let modifier_result =
                        Self::eval_modifier(&mut single, sides, results, pair, source, &modifier)?;
                    results = modifier_result.results;
                    modifier = match modifier_result.modifier {
                        dice::Modifier::TargetDoubleFailure(t, f, d) => match modifier {
                            dice::Modifier::TargetDoubleFailure(ot, of, od) => {
                                if t > 0 {
                                    dice::Modifier::TargetDoubleFailure(t, of, od)
                                } else if f > 0 {
                                    dice::Modifier::TargetDoubleFailure(ot, f, od)
                                } else {
                                    dice::Modifier::TargetDoubleFailure(ot, of, d)
                                }
                            }
                            _ => {
                                single.add_history(results.clone(), is_fudge);
                                modifier_result.modifier
                            }
                        },
                        dice::Modifier::TargetEnum(_) => {
                            single.add_history(results.clone(), is_fudge);
                            modifier_result.modifier
                        }
                        _ => modifier_result.modifier,
                    };
                    maybe_modifier = dice.next();
                }
            } else {
                single.add_history(results, is_fudge);
            }
            single.eval_total(modifier)?;
        } else {
            single.add_history(results, is_fudge);
            single.eval_total(if is_fudge {
                dice::Modifier::Fudge
            } else {
                dice::Modifier::None(Rule::expr)
            })?;
        }
        Ok(single)
    }

    // compute a whole roll expression
    pub(crate) fn eval<S: roll::Source>(
        expr: Pairs<Rule>,
        source: &mut S,
        is_block: bool,
    ) -> Result<kind::Single> {
        let result = C.raise(
            expr,
            |pair: Pair<Rule>| match pair.as_rule() {
                Rule::integer => Ok(kind::Single::with_total(
                    pair.as_str().replace(' ', "").parse::<i64>().unwrap(),
                )),
                Rule::float => Ok(kind::Single::with_float(
                    pair.as_str().replace(' ', "").parse::<f64>().unwrap(),
                )),
                Rule::block_expr => {
                    let expr = pair.into_inner().next().unwrap().into_inner();
                    Self::eval(expr, source, true)
                }
                Rule::dice => Self::eval_roll(pair.into_inner(), source),
                _ => unreachable!("{:#?}", pair),
            },
            |lhs: Result<kind::Single>, op: Pair<Rule>, rhs: Result<kind::Single>| match (lhs, rhs)
            {
                (Ok(lhs), Ok(rhs)) => match op.as_rule() {
                    Rule::add => Ok(lhs + rhs),
                    Rule::sub => Ok(lhs - rhs),
                    Rule::mul => Ok(lhs * rhs),
                    Rule::div => {
                        if rhs.is_zero() {
                            Err("can't divide by zero".into())
                        } else {
                            Ok(lhs / rhs)
                        }
                    }
                    _ => unreachable!(),
                },
                (Err(error), _) => Err(error),
                (_, Err(error)) => Err(error),
            },
        );
        match result {
            Ok(mut result) => {
                if is_block {
                    result.add_parens();
                }
                Ok(result)
            }
            error @ Err(_) => error,
        }
    }

    pub(crate) fn roll<S: roll::Source>(
        amount: u64,
        sides: u64,
        source: &mut S,
    ) -> Vec<dice::Result> {
        (0..amount)
            .map(|_| dice::Result::new(source.throw(sides), sides))
            .collect()
    }
}
