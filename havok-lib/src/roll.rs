pub mod history;
pub mod kind;

/// Keep the roll expression type
#[derive(Debug, Clone)]
pub enum Kind {
    Single(kind::Single),
    Multi(kind::Multi),
}

/// Keep a roll expression result
#[derive(Debug, Clone)]
pub struct Result {
    result: Kind,
    reason: Option<String>,
}

impl Result {
    /// New with single roll expression
    pub fn new_single(r: kind::Single) -> Self {
        Result {
            result: Kind::Single(r),
            reason: None,
        }
    }

    /// New with multi roll expression
    pub fn new_multi(v: Vec<kind::Single>, total: Option<i64>) -> Self {
        Result {
            result: Kind::Multi(kind::Multi { rolls: v, total }),
            reason: None,
        }
    }

    pub fn add_reason(&mut self, reason: String) {
        self.reason = Some(reason);
    }

    pub fn get_reason(&self) -> Option<&String> {
        self.reason.as_ref()
    }

    pub fn get_result(&self) -> &Kind {
        &self.result
    }

    /// Check and return result as single roll expression
    pub fn as_single(&self) -> Option<&kind::Single> {
        match &self.result {
            Kind::Single(result) => Some(result),
            Kind::Multi(_) => None,
        }
    }

    /// Check and return result as multi roll expression
    pub fn as_multi(&self) -> Option<&kind::Multi> {
        match &self.result {
            Kind::Single(_) => None,
            Kind::Multi(results) => Some(results),
        }
    }
}

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.result {
            Kind::Single(single) => {
                write!(f, "{}", single.to_string(true))?;
                if let Some(reason) = &self.reason {
                    write!(f, " *reason* `{}`", reason)?;
                }
            }
            Kind::Multi(multi) => match multi.get_total() {
                Some(total) => {
                    (*multi)
                        .iter()
                        .try_for_each(|result| writeln!(f, "`{}`", result.to_string_history()))?;
                    write!(f, "*total* **{}**", total)?;
                    if let Some(reason) = &self.reason {
                        write!(f, " *reason* `{}`", reason)?;
                    }
                }
                None => {
                    (*multi)
                        .iter()
                        .try_for_each(|result| writeln!(f, "{}", result.to_string(true)))?;
                    if let Some(reason) = &self.reason {
                        write!(f, "*reason* `{}`", reason)?;
                    }
                }
            },
        }
        Ok(())
    }
}

/// Interface for rolling dices
pub trait Source {
    fn throw(&mut self, sides: u64) -> u64;
}
