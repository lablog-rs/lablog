use clap::ArgMatches;
use errors::*;
use log::LogLevel;
use std::path::PathBuf;

mod errors {
    error_chain!{}
}

#[derive(Debug)]
pub struct Options {
    pub source_dir: PathBuf,
    pub destination_dir: PathBuf,
    pub loglevel: LogLevel,
}

impl Options {
    pub fn try_from(matches: &ArgMatches) -> Result<Self> {
        let loglevel: LogLevel = value_t!(matches, "loglevel", LogLevel).chain_err(|| "can not parse loglevel from args")?;

        let source_dir: PathBuf = {
            let arg = matches
                .value_of("source_dir")
                .chain_err(|| "can not get source_dir from args")?;

            PathBuf::from(arg)
        };

        let destination_dir: PathBuf = {
            let arg = matches
                .value_of("destination_dir")
                .chain_err(|| "can not get destination_dir from args")?;

            PathBuf::from(arg)
        };

        let options = Options {
            loglevel: loglevel,
            source_dir: source_dir,
            destination_dir: destination_dir,
        };

        Ok(options)
    }
}
