use clap::crate_authors;
use clap::crate_description;
use clap::crate_version;
pub(crate) use clap::Parser;

const HAVOK_AUTHOR: &str = crate_authors!();
const HAVOK_VERSION: &str = crate_version!();
const HAVOK_ABOUT: &str = crate_description!();
const HAVOK_FLAG_V_SHORT: char = 'v';
const HAVOK_FLAG_V_HELP: &str = "Enable verbose logging";

#[derive(Parser, Debug)]
#[command(author = HAVOK_AUTHOR, version = HAVOK_VERSION, about = HAVOK_ABOUT)]
pub(crate) struct Args {
    #[arg(short = HAVOK_FLAG_V_SHORT, long, help = HAVOK_FLAG_V_HELP, action)]
    pub(crate) verbose: bool,
}
