#[macro_use] extern crate colour;
#[macro_use] extern crate prettytable;

use structopt::StructOpt;
use structopt_flags::{LogLevel, LogLevelOpt};

const APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
);

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(flatten)]
    log_level: LogLevelOpt,

    #[structopt(flatten)]
    command: Subcommands
}

#[derive(Debug, StructOpt)]
enum Subcommands {
    Offers {
        #[structopt(short, long, default_value = "50")]
        price: u16,
    
        #[structopt(short, long, default_value = "5")]
        discount: u16,
    },

    Deal {
        #[structopt(short, long)]
        stop_when_found: bool       
    }
}

mod offers;
mod deal;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let Opt { log_level, command } = Opt::from_args();

    simplelog::TermLogger::init(
        log_level.get_level_filter(),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout
    ).ok();

    match command {
        Subcommands::Offers { price, discount } => offers::list(price, discount).await?,
        Subcommands::Deal { stop_when_found } => deal::find(stop_when_found).await?,
    };

    Ok(())
}