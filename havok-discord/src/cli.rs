use clap::crate_authors;
use clap::crate_description;
use clap::crate_version;
pub use clap::Parser;

const HAVOK_AUTHOR: &str = crate_authors!();
const HAVOK_VERSION: &str = crate_version!();
const HAVOK_ABOUT: &str = crate_description!();
const HAVOK_FLAG_D_SHORT: char = 'd';
const HAVOK_FLAG_D_HELP: &str = "Enable Debug logging";

#[derive(Parser, Debug)]
#[command(author = HAVOK_AUTHOR, version = HAVOK_VERSION, about = HAVOK_ABOUT)]
pub struct Args {
    #[arg(short = HAVOK_FLAG_D_SHORT, long, help = HAVOK_FLAG_D_HELP, action)]
    pub debug: bool,
}
