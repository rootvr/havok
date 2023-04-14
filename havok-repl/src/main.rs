mod helper;
use helper::ReplHelper;

use colored::Colorize;
use havok_lib::solver::Solver;
use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::CompletionType;
use rustyline::Config;
use rustyline::EditMode;
use rustyline::Editor;
use rustyline::Result;
use termimad::crossterm::style::Color;
use termimad::MadSkin;

const HISTORY_FILE: &str = "history";

fn main() -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();
    let helper = ReplHelper {
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
        let prompt = format!("repl: {}> ", count);
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
