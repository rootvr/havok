use crate::constant;
use crate::dice;
use crate::error::Result;
use crate::roll::history::History;
use crate::roll::kind;
use std::ops::Deref;

fn merge_history(lhs: &mut Single, rhs: &mut Single, oper: &'static str) {
    if !rhs.history.is_empty() {
        lhs.history.push(History::Operator(oper));
        lhs.history.append(&mut rhs.history);
    }
}

/// Represents a single roll with the history of steps taken
#[derive(Debug, Clone)]
pub struct Single {
    /// With modifier `t` or `f`: successes - failures
    total: i64,
    /// dummy flag to avoid re-computing a total
    dirty: bool,
    constant: Option<f64>,
    history: Vec<History>,
}

impl Single {
    pub fn new() -> Self {
        Self {
            total: 0,
            dirty: true,
            constant: None,
            history: Vec::new(),
        }
    }

    /// New with already a total
    pub fn with_total(total: i64) -> Self {
        Self {
            total,
            dirty: false,
            constant: None,
            history: vec![History::Constant(constant::Constant::Integer(total))],
        }
    }

    /// New with already a total that contains a float constant
    pub fn with_float(f: f64) -> Self {
        Self {
            total: f as i64,
            dirty: false,
            constant: Some(f),
            history: vec![History::Constant(constant::Constant::Float(f))],
        }
    }

    pub fn get_history(&self) -> &Vec<History> {
        &self.history
    }

    /// Add a step in the history
    pub fn add_history(&mut self, mut history: Vec<dice::Result>, is_fudge: bool) {
        self.dirty = true;
        history.sort_unstable_by(|a, b| b.cmp(a));
        self.history.push(if is_fudge {
            History::Fudge(history.iter().map(|r| r.value).collect())
        } else {
            History::Roll(history)
        });
    }

    pub fn add_parens(&mut self) {
        self.history.insert(0, History::OpenParen);
        self.history.push(History::CloseParen);
    }

    /// Evaluate the total value according to some modifier
    pub fn eval_total(&mut self, modifier: dice::Modifier) -> Result<i64> {
        if self.dirty {
            self.dirty = false;
            let mut flat = self.history.iter().fold(Vec::new(), |mut acc, h| {
                match h {
                    History::Roll(r) => {
                        let mut c = r.iter().map(|u| u.value as i64).collect();
                        acc.append(&mut c);
                    }
                    History::Fudge(r) => {
                        let mut c = r.iter().map(|u| *u as i64).collect();
                        acc.append(&mut c);
                    }
                    History::Constant(v) => acc.push(v.get_value()),
                    _ => (),
                };
                acc
            });
            flat.sort_unstable();
            let flat = flat;
            match modifier {
                dice::Modifier::KeepHigh(n)
                | dice::Modifier::KeepLow(n)
                | dice::Modifier::DropHigh(n)
                | dice::Modifier::DropLow(n) => {
                    if n > flat.len() {
                        return Err("Not enough dice to keep or drop".into());
                    }
                }
                dice::Modifier::None(_)
                | dice::Modifier::TargetDoubleFailure(_, _, _)
                | dice::Modifier::TargetEnum(_)
                | dice::Modifier::Fudge => (),
            }
            let slice = match modifier {
                dice::Modifier::KeepHigh(n) => &flat[flat.len() - n..],
                dice::Modifier::KeepLow(n) => &flat[..n],
                dice::Modifier::DropHigh(n) => &flat[..flat.len() - n],
                dice::Modifier::DropLow(n) => &flat[n..],
                dice::Modifier::None(_)
                | dice::Modifier::TargetDoubleFailure(_, _, _)
                | dice::Modifier::TargetEnum(_)
                | dice::Modifier::Fudge => flat.as_slice(),
            };
            self.total = match modifier {
                dice::Modifier::TargetDoubleFailure(t, f, d) => slice.iter().fold(0, |acc, &x| {
                    let x = x as u64;
                    if d > 0 && x >= d {
                        acc + 2
                    } else if t > 0 && x >= t {
                        acc + 1
                    } else if f > 0 && x <= f {
                        acc - 1
                    } else {
                        acc
                    }
                }),
                dice::Modifier::TargetEnum(v) => slice.iter().fold(0, |acc, &x| {
                    if v.contains(&(x as u64)) {
                        acc + 1
                    } else {
                        acc
                    }
                }),
                dice::Modifier::Fudge => slice.iter().fold(0, |acc, &x| {
                    if x <= 2 {
                        acc - 1
                    } else if x <= 4 {
                        acc
                    } else {
                        acc + 1
                    }
                }),
                _ => slice.iter().sum::<i64>(),
            };
        }
        Ok(self.total)
    }

    pub fn get_total(&self) -> i64 {
        self.total
    }

    /// Check if optional constant or total is 0
    pub fn is_zero(&self) -> bool {
        if let Some(c) = self.constant {
            c == 0.0
        } else {
            self.total == 0
        }
    }

    /// Stringify history
    pub fn to_string_history(&self) -> String {
        self.history.iter().fold(String::new(), |mut s, v| {
            s.push_str(v.to_string().as_str());
            s
        })
    }

    /// Stringify self with(out) markdown formatting
    pub fn to_string(&self, md: bool) -> String {
        if self.history.is_empty() {
            if md {
                format!("`{}`", self.total)
            } else {
                format!("{}", self.total)
            }
        } else {
            let s = self.to_string_history();
            format!(
                "{1}{0}{1} = {2}{3}{2}",
                s,
                if md { "`" } else { "" },
                if md { "**" } else { "" },
                self.get_total()
            )
        }
    }
}

impl std::ops::Add for Single {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        merge_history(&mut self, &mut rhs, " + ");
        let total = match (self.constant, rhs.constant) {
            (None, None) => self.total + rhs.total,
            (None, Some(c)) => (self.total as f64 + c).trunc() as i64,
            (Some(c), None) => (c + rhs.total as f64).trunc() as i64,
            (Some(l), Some(r)) => (l + r).trunc() as i64,
        };
        Single {
            total,
            dirty: false,
            constant: None,
            history: self.history,
        }
    }
}

impl std::ops::Sub for Single {
    type Output = Self;

    fn sub(mut self, mut rhs: Self) -> Self::Output {
        merge_history(&mut self, &mut rhs, " - ");
        let total = match (self.constant, rhs.constant) {
            (None, None) => self.total - rhs.total,
            (None, Some(c)) => (self.total as f64 - c).trunc() as i64,
            (Some(c), None) => (c - rhs.total as f64).trunc() as i64,
            (Some(l), Some(r)) => (l - r).trunc() as i64,
        };
        Single {
            total,
            dirty: false,
            constant: None,
            history: self.history,
        }
    }
}

impl std::ops::Mul for Single {
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self::Output {
        merge_history(&mut self, &mut rhs, " * ");
        let total = match (self.constant, rhs.constant) {
            (None, None) => self.total * rhs.total,
            (None, Some(c)) => (self.total as f64 * c).trunc() as i64,
            (Some(c), None) => (c * rhs.total as f64).trunc() as i64,
            (Some(l), Some(r)) => (l * r).trunc() as i64,
        };
        Single {
            total,
            dirty: false,
            constant: None,
            history: self.history,
        }
    }
}

impl std::ops::Div for Single {
    type Output = Self;

    fn div(mut self, mut rhs: Self) -> Self::Output {
        merge_history(&mut self, &mut rhs, " / ");
        let total = match (self.constant, rhs.constant) {
            (None, None) => self.total / rhs.total,
            (None, Some(c)) => (self.total as f64 / c).trunc() as i64,
            (Some(c), None) => (c / rhs.total as f64).trunc() as i64,
            (Some(l), Some(r)) => (l / r).trunc() as i64,
        };
        Single {
            total,
            dirty: false,
            constant: None,
            history: self.history,
        }
    }
}

impl std::fmt::Display for Single {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(true))?;
        Ok(())
    }
}

/// Represents a multi roll
#[derive(Debug, Clone)]
pub struct Multi {
    pub total: Option<i64>,
    pub rolls: Vec<kind::Single>,
}

impl Multi {
    pub fn get_total(&self) -> Option<i64> {
        self.total
    }
}

impl Deref for Multi {
    type Target = Vec<kind::Single>;

    fn deref(&self) -> &Self::Target {
        &self.rolls
    }
}
