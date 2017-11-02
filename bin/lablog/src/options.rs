use clap::ArgMatches;
use errors::*;
use log::LogLevel;
use std::path::PathBuf;
use xdg::BaseDirectories;

#[derive(Debug)]
pub struct Options {
    pub datadir: PathBuf,
    pub loglevel: LogLevel,
}

impl Options {
    pub fn try_from(matches: &ArgMatches) -> Result<Self> {
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

        let loglevel: LogLevel = value_t!(matches, "loglevel", LogLevel).chain_err(
            || "can not parse loglevel from args",
        )?;

        let options = Options {
            datadir: datadir,
            loglevel: loglevel,
        };
        Ok(options)
    }
}
