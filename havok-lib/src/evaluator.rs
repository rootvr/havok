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
    pub const MAX_DICE_AMOUNT: u64 = 5000;
    pub const MAX_DICE_SIDES: u64 = 5000;
}

/// Represent an evaluator
pub struct Evaluator;

impl Evaluator {
    fn eval_explode<S: roll::Source>(
        rolls: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        option: Pair<Rule>,
        prior: &dice::Modifier,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(option).unwrap_or(sides);
        let nb = results.iter().filter(|x| x.value >= value).count() as u64;
        if prior != &dice::Modifier::None(Rule::explode)
            && prior != &dice::Modifier::None(Rule::i_explode)
        {
            rolls.add_history(results.clone(), false);
        }
        let res = if nb > 0 {
            let res = Self::roll(nb, sides, source);
            rolls.add_history(res.clone(), false);
            res
        } else {
            results
        };
        (dice::Modifier::None(Rule::explode), res)
    }

    fn eval_indef_explode<S: roll::Source>(
        rolls: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        option: Pair<Rule>,
        prior: &dice::Modifier,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(option).unwrap_or(sides);
        if prior != &dice::Modifier::None(Rule::explode)
            && prior != &dice::Modifier::None(Rule::i_explode)
        {
            rolls.add_history(results.clone(), false);
        }
        let mut nb = results.into_iter().filter(|x| x.value >= value).count() as u64;
        let mut res = Vec::new();
        while nb > 0 {
            res = Self::roll(nb, sides, source);
            nb = res.iter().filter(|x| x.value >= value).count() as u64;
            rolls.add_history(res.clone(), false);
        }
        (dice::Modifier::None(Rule::i_explode), res)
    }

    fn eval_reroll<S: roll::Source>(
        rolls: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        option: Pair<Rule>,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(option).unwrap();
        let mut has_rerolled = false;
        let res: Vec<dice::Result> = results
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
            rolls.add_history(res.clone(), false);
        }
        (dice::Modifier::None(Rule::reroll), res)
    }

    fn eval_indef_reroll<S: roll::Source>(
        rolls: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        option: Pair<Rule>,
        source: &mut S,
    ) -> (dice::Modifier, Vec<dice::Result>) {
        let value = Parser::extract_modifier_value(option).unwrap();
        let mut has_rerolled = false;
        let res: Vec<dice::Result> = results
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
            rolls.add_history(res.clone(), false);
        }
        (dice::Modifier::None(Rule::i_reroll), res)
    }

    fn eval_modifier<S: roll::Source>(
        rolls: &mut kind::Single,
        sides: u64,
        results: Vec<dice::Result>,
        option: Pair<Rule>,
        source: &mut S,
        prev: &dice::Modifier,
    ) -> Result<dice::modifier::Result> {
        let (modifier, mut res) = match &option.as_rule() {
            Rule::explode => Self::eval_explode(rolls, sides, results, option, prev, source),
            Rule::i_explode => {
                Self::eval_indef_explode(rolls, sides, results, option, prev, source)
            }
            Rule::reroll => Self::eval_reroll(rolls, sides, results, option, source),
            Rule::i_reroll => Self::eval_indef_reroll(rolls, sides, results, option, source),
            Rule::keep_hi => {
                let value = Parser::extract_modifier_value(option).unwrap();
                if rolls.get_history().is_empty() {
                    rolls.add_history(results.clone(), false);
                }
                (dice::Modifier::KeepHigh(value as usize), results)
            }
            Rule::keep_lo => {
                let value = Parser::extract_modifier_value(option).unwrap();
                if rolls.get_history().is_empty() {
                    rolls.add_history(results.clone(), false);
                }
                (dice::Modifier::KeepLow(value as usize), results)
            }
            Rule::drop_hi => {
                let value = Parser::extract_modifier_value(option).unwrap();
                if rolls.get_history().is_empty() {
                    rolls.add_history(results.clone(), false);
                }
                (dice::Modifier::DropHigh(value as usize), results)
            }
            Rule::drop_lo => {
                let value = Parser::extract_modifier_value(option).unwrap();
                if rolls.get_history().is_empty() {
                    rolls.add_history(results.clone(), false);
                }
                (dice::Modifier::DropLow(value as usize), results)
            }
            Rule::target => {
                let value_or_enum = option.into_inner().next().unwrap();
                match value_or_enum.as_rule() {
                    Rule::number => (
                        dice::Modifier::TargetDoubleFailure(
                            value_or_enum.as_str().parse::<u64>().unwrap(),
                            0,
                            0,
                        ),
                        results,
                    ),
                    Rule::target_enum => {
                        let numbers_list = value_or_enum.into_inner();
                        let numbers_list: Vec<_> = numbers_list
                            .map(|p| p.as_str().parse::<u64>().unwrap())
                            .collect();
                        (dice::Modifier::TargetEnum(numbers_list), results)
                    }
                    _ => unreachable!(),
                }
            }
            Rule::double_target => {
                let value = Parser::extract_modifier_value(option).unwrap();
                (dice::Modifier::TargetDoubleFailure(0, 0, value), results)
            }
            Rule::failure => {
                let value = Parser::extract_modifier_value(option).unwrap();
                (dice::Modifier::TargetDoubleFailure(0, value, 0), results)
            }
            _ => unreachable!("{:#?}", option),
        };
        let n = match modifier {
            dice::Modifier::KeepHigh(n) | dice::Modifier::KeepLow(n) => {
                if n > res.len() {
                    res.len()
                } else {
                    n
                }
            }
            dice::Modifier::DropHigh(n) | dice::Modifier::DropLow(n) => {
                if n > res.len() {
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
        res.sort_unstable();
        let res = match modifier {
            dice::Modifier::KeepHigh(_) => res[res.len() - n..].to_vec(),
            dice::Modifier::KeepLow(_) => res[..n].to_vec(),
            dice::Modifier::DropHigh(_) => res[..res.len() - n].to_vec(),
            dice::Modifier::DropLow(_) => res[n..].to_vec(),
            dice::Modifier::None(_)
            | dice::Modifier::TargetDoubleFailure(_, _, _)
            | dice::Modifier::TargetEnum(_)
            | dice::Modifier::Fudge => res,
        };
        Ok(dice::modifier::Result {
            results: res,
            modifier,
        })
    }

    fn eval_roll<S: roll::Source>(mut dice: Pairs<Rule>, source: &mut S) -> Result<kind::Single> {
        let mut rolls = kind::Single::new();
        let maybe_nb = dice.next().unwrap();
        let nb = match maybe_nb.as_rule() {
            Rule::nb_dice => {
                dice.next(); // skip `d` token
                let n = maybe_nb.as_str().parse::<u64>().unwrap();
                if n > limits::MAX_DICE_AMOUNT {
                    return Err(format!(
                        "exceeded max allowed amount of dices `{}`",
                        limits::MAX_DICE_AMOUNT
                    )
                    .into());
                }
                n
            }
            Rule::roll => 1, // no number before `d`, assume 1 dice
            _ => unreachable!("{:?}", maybe_nb),
        };
        let pair = dice.next().unwrap();
        let (sides, is_fudge) = match pair.as_rule() {
            Rule::number => (pair.as_str().parse::<u64>().unwrap(), false),
            Rule::fudge => (6, true),
            _ => unreachable!("{:?}", pair),
        };
        if sides == 0 {
            return Err("invalid `0` sides dice provided".into());
        } else if sides > limits::MAX_DICE_SIDES {
            return Err(format!(
                "exceeded max allowed number of dice sides `{}`",
                limits::MAX_DICE_SIDES
            )
            .into());
        }
        let mut res = Self::roll(nb, sides, source);
        let mut modifier = dice::Modifier::None(Rule::expr);
        let mut next_option = dice.next();
        if !is_fudge {
            if next_option.is_some() {
                while next_option.is_some() {
                    let option = next_option.unwrap();
                    let opt_res =
                        Self::eval_modifier(&mut rolls, sides, res, option, source, &modifier)?;
                    res = opt_res.results;
                    modifier = match opt_res.modifier {
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
                                rolls.add_history(res.clone(), is_fudge);
                                opt_res.modifier
                            }
                        },
                        dice::Modifier::TargetEnum(_) => {
                            rolls.add_history(res.clone(), is_fudge);
                            opt_res.modifier
                        }
                        _ => opt_res.modifier,
                    };
                    next_option = dice.next();
                }
            } else {
                rolls.add_history(res, is_fudge);
            }
            rolls.eval_total(modifier)?;
        } else {
            rolls.add_history(res, is_fudge);
            rolls.eval_total(if is_fudge {
                dice::Modifier::Fudge
            } else {
                dice::Modifier::None(Rule::expr)
            })?;
        }
        Ok(rolls)
    }

    // compute a whole roll expression
    pub fn eval<S: roll::Source>(
        expr: Pairs<Rule>,
        source: &mut S,
        is_block: bool,
    ) -> Result<kind::Single> {
        let res = C.raise(
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
                            Err("Can't divide by zero".into())
                        } else {
                            Ok(lhs / rhs)
                        }
                    }
                    _ => unreachable!(),
                },
                (Err(e), _) => Err(e),
                (_, Err(e)) => Err(e),
            },
        );
        match res {
            Ok(mut single_roll_res) => {
                if is_block {
                    single_roll_res.add_parens();
                }
                Ok(single_roll_res)
            }
            e @ Err(_) => e,
        }
    }

    pub fn roll<S: roll::Source>(amount: u64, sides: u64, source: &mut S) -> Vec<dice::Result> {
        (0..amount)
            .map(|_| dice::Result::new(source.throw(sides), sides))
            .collect()
    }
}
