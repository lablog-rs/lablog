#[macro_use]
extern crate log;
extern crate loggerv;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

mod options;

mod errors {
    error_chain!{}
}

use clap::App;
use errors::*;
use options::Options;

fn main() {
    if let Err(e) = run() {
        error!("error while running: {}", e);
        for e in e.iter().skip(1) {
            error!("caused by: {}", e);
        }

        if let Some(backtrace) = e.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    let options = Options::try_from(&matches).chain_err(
        || "can not get options from matches",
    )?;

    loggerv::init_with_level(options.loglevel).chain_err(
        || "can not initialize logger with parsed loglevel",
    )?;

    trace!("matches: {:#?}", matches);
    trace!("options: {:#?}", options);

    Ok(())
}
