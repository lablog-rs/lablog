#[macro_use]
extern crate log;
extern crate loggerv;

#[macro_use]
extern crate clap;

extern crate xdg;

extern crate lablog_store as store;
extern crate lablog_store_csv as store_csv;

use clap::App;
use clap::ArgMatches;
use log::LogLevel;
use std::path::PathBuf;
use store::Store;
use store::errors::*;
use store_csv::*;
use xdg::BaseDirectories;

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

    // setup loglevel
    {
        let loglevel: LogLevel = value_t!(matches, "loglevel", LogLevel).chain_err(
            || "can not parse loglevel from args",
        )?;
        loggerv::init_with_level(loglevel).chain_err(
            || "can not initialize logger with parsed loglevel",
        )?;
    }
    trace!("matches: {:#?}", matches);

    let options = Options::try_from(&matches).chain_err(
        || "can not get options from matches",
    )?;
    trace!("options: {:#?}", options);

    match matches.subcommand_name() {
        Some("projects") => {
            run_projects(options).chain_err(|| "problem while running projects subcommand")
        }
        Some("note") => {
            run_note(matches.subcommand_matches("note").unwrap())
                .chain_err(|| "problem while running note subcommand")
        }
        _ => unreachable!(),
    }
}

fn run_projects(options: Options) -> Result<()> {
    let store = CSVStore::new(options.datadir);
    let projects = store.get_projects().chain_err(
        || "can not get projects from store",
    )?;

    for project in projects {
        if project.archived {
            continue;
        }
        println!("{}", project);
    }

    Ok(())
}

fn run_note(matches: &ArgMatches) -> Result<()> {
    trace!("run_note matches: {:#?}", matches);
    println!("loglevel: {:#?}", matches.value_of("loglevel"));

    match matches.subcommand_name() {
        Some("editor") => unimplemented!(),
        Some("file") => unimplemented!(),
        Some("text") => unimplemented!(),
        _ => unreachable!(),
    }
}

#[derive(Debug)]
struct Options {
    datadir: PathBuf,
}

impl Options {
    fn try_from(matches: &ArgMatches) -> Result<Self> {
        let datadir = match matches.value_of("datadir").unwrap() {
            "$XDG_DATA_HOME/lablog" => {
                let xdg = BaseDirectories::new().chain_err(
                    || "can not get xdg base directory",
                )?;

                xdg.create_data_directory("lablog").chain_err(
                    || "can not create xdg base directory",
                )?
            }
            _ => {
                PathBuf::from(matches.value_of("datadir").ok_or(
                    "can not parse datadir from args",
                )?)
            }
        };

        let options = Options { datadir: datadir };
        Ok(options)
    }
}
