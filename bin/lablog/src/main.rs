#[macro_use]
extern crate log;
extern crate loggerv;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

extern crate xdg;

extern crate lablog_store as store;
extern crate lablog_store_csv as store_csv;

extern crate tempdir;

extern crate regex;

mod errors;
mod formatter;
mod helper;
mod options;

use clap::App;
use clap::ArgMatches;
use errors::*;
use options::Options;
use regex::Regex;
use std::io;
use store::ProjectName;
use store::Store;
use store_csv::*;

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

    match matches.subcommand_name() {
        Some("projects") => {
            run_projects(options).chain_err(|| "problem while running projects subcommand")
        }
        Some("notes") => {
            run_notes(matches.subcommand_matches("notes").unwrap(), options)
                .chain_err(|| "problem while running notes subcommand")
        }
        Some("note") => {
            run_note(matches.subcommand_matches("note").unwrap(), options)
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

    trace!("projects: {:#?}", projects);

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for project in projects {
        if project.archived {
            continue;
        }

        formatter::format_project_name(&mut handle, &project.name)
            .chain_err(|| "can not format project name")?;
    }

    Ok(())
}

fn run_notes(matches: &ArgMatches, options: Options) -> Result<()> {
    let store = CSVStore::new(options.datadir);

    let projects = store.get_projects().chain_err(
        || "can not get projects from store",
    )?;

    let filter = matches.value_of("filter").chain_err(
        || "can not get regex filter for notes filtering",
    )?;

    let regex = Regex::new(filter).chain_err(
        || "can not create regex out of filter argument",
    )?;

    let projects: store::Projects = projects
        .into_iter()
        .filter(|project| regex.is_match((&project.name).into()))
        .collect();

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for project in projects {
        if project.archived {
            continue;
        }

        formatter::format_project_name(&mut handle, &project.name)
            .chain_err(|| "can not format project name")?;

        for note in project.notes {
            formatter::format_note(&mut handle, &note).chain_err(
                || "can not format note",
            )?
        }
    }

    Ok(())
}

fn run_note(matches: &ArgMatches, options: Options) -> Result<()> {
    match matches.subcommand_name() {
        Some("editor") => {
            // TODO: find out if we can move the project to a global arg of the subcommand,
            // had problems with clap complaining that it would clash with the loglevel
            // argument or something
            let submatches = matches.subcommand_matches("editor").unwrap();
            trace!("editor submatches: {:#?}", submatches);

            let project_name = value_t!(submatches, "project", store::ProjectName)
                .chain_err(|| "can not get project name to write note to")?;
            trace!("project_name: {:#?}", project_name);

            run_note_editor(options, &project_name).chain_err(
                || "problem while running editor subcommand",
            )
        }
        Some("file") | Some("text") => bail!("unimplemented"),
        _ => unreachable!(),
    }
}

fn run_note_editor(options: Options, project_name: &ProjectName) -> Result<()> {
    let store = CSVStore::new(options.datadir);

    let note = helper::string_from_editor(None).chain_err(
        || "can not get note from running the editor",
    )?;

    store.write_note(project_name, &note.into()).chain_err(
        || "can not write note into store",
    )
}
