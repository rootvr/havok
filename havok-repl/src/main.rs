use colored::Colorize;
use havok_lib::solver::Solver;
use rustyline::completion::Completer;
use rustyline::completion::FilenameCompleter;
use rustyline::completion::Pair;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::Hinter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::validate::ValidationContext;
use rustyline::validate::ValidationResult;
use rustyline::validate::Validator;
use rustyline::CompletionType;
use rustyline::Config;
use rustyline::Context;
use rustyline::EditMode;
use rustyline::Editor;
use rustyline::Result;
use rustyline_derive::Helper;
use std::borrow::Cow;
use std::borrow::Cow::Borrowed;
use std::borrow::Cow::Owned;
use termimad::crossterm::style::Color;
use termimad::MadSkin;

#[derive(Helper)]
struct HavokReplHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    _validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored: String,
}

impl Completer for HavokReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for HavokReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for HavokReplHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(hint.dimmed().to_string())
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for HavokReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        let _ = ctx;
        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

const HISTORY_FILE: &str = "history";

fn main() -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let helper = HavokReplHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        _validator: MatchingBracketValidator::new(),
        hinter: HistoryHinter {},
        colored: "".to_owned(),
    };
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Color::Yellow);
    skin.inline_code.set_fg(Color::Magenta);
    let mut rline = Editor::with_config(config)?;
    rline.set_helper(Some(helper));
    if rline.load_history(HISTORY_FILE).is_err() {
        eprintln!("{}", "repl: warn: no previous history".bold().yellow());
    }
    let mut count = 1u64;
    loop {
        let prompt = format!("repl:{}> ", count);
        rline.helper_mut().expect("repl: panic: no helper").colored =
            prompt.bold().green().to_string();
        let readline = rline.readline(&prompt);
        match readline {
            Ok(line) => {
                rline.add_history_entry(line.as_str())?;
                if !line.is_empty() {
                    match Solver::new(line.as_str().trim()).unwrap().solve() {
                        Ok(result) => println!("{}", skin.inline(&format!("{}", result).magenta())),
                        Err(error) => eprintln!("{}", format!("{}", error).bold().red()),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("{}", "repl: signal: CTRL-C".bold().yellow());
                break;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("{}", "repl: signal: CTRL-D".bold().yellow());
                break;
            }
            Err(error) => {
                eprintln!("{}", format!("repl: error: `{:?}`", error).bold().red());
                break;
            }
        }
        count += 1;
    }
    rline.append_history(HISTORY_FILE)
}
