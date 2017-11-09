#[macro_use]
extern crate log;
extern crate loggerv;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

extern crate lablog_store_csv as store_csv;

mod errors;
mod options;
mod v2;
mod v3;

use clap::App;
use errors::*;
use options::Options;
use store_csv::CSVStore;

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

    let options = Options::try_from(&matches).chain_err(|| "can not get options from matches")?;

    loggerv::init_with_level(options.loglevel).chain_err(|| "can not initialize logger with parsed loglevel")?;

    trace!("matches: {:#?}", matches);
    trace!("options: {:#?}", options);

    let old_notes = v2::read_notes(&options.source_dir);

    trace!("old_notes: {:#?}", old_notes);

    let store = CSVStore::new(options.destination_dir);

    v3::write_v2_notes(&store, old_notes).chain_err(|| "can not write v2 notes into v3 store")?;

    Ok(())
}
