use chrono::{
    DateTime,
    Duration,
    Local,
    ParseResult,
    TimeZone,
    Utc,
};
use errors::*;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use store::project::Projects;
use tempdir::TempDir;

pub fn string_from_editor(prepoluate: Option<&str>) -> Result<String> {
    let tmpdir = TempDir::new("lablog_tmp").unwrap();
    let tmppath = tmpdir.path().join("note.asciidoc");
    let editor = {
        match env::var("VISUAL") {
            Ok(editor) => editor,
            Err(_) => match env::var("EDITOR") {
                Ok(editor) => editor,
                Err(_) => bail!("not editor set. either set $VISUAL OR $EDITOR environment variable"),
            },
        }
    };

    if let Some(content) = prepoluate {
        let mut file = File::create(tmppath.display().to_string()).chain_err(|| "can not open tmp editor file to prepoluate with string")?;

        file.write_all(content.as_bytes())
            .chain_err(|| "can not prepoluate editor tmp file")?;
    }

    let mut editor_command = Command::new(editor);
    editor_command.arg(tmppath.display().to_string());

    editor_command
        .spawn()
        .chain_err(|| "couldn not launch editor")?
        .wait()
        .chain_err(|| "problem while running editor")?;

    let mut string = String::new();
    let mut file = File::open(tmppath).chain_err(|| "can not open tmppath for reading")?;

    file.read_to_string(&mut string)
        .chain_err(|| "can not read tmpfile to string")?;

    Ok(string)
}

pub fn try_multiple_time_parser(input: &str) -> ParseResult<DateTime<Utc>> {
    let input = match input {
        "today" => format!("{}", Local::now().format("%Y-%m-%d")),
        "yesterday" => {
            let yesterday = Local::now() - Duration::days(1);
            format!("{}", yesterday.format("%Y-%m-%d"))
        }
        _ => String::from(input),
    };

    trace!("time_parser input after natural timestamp: {}", input);

    input
        .parse()
        .or(Utc.datetime_from_str(input.as_str(), "%Y-%m-%d %H:%M:%S"))
        .or(Utc.datetime_from_str(
            format!("{}:00", input).as_str(),
            "%Y-%m-%d %H:%M:%S",
        ))
        .or(Utc.datetime_from_str(
            format!("{}:00:00", input).as_str(),
            "%Y-%m-%d %H:%M:%S",
        ))
        .or(Utc.datetime_from_str(
            format!("{} 00:00:00", input).as_str(),
            "%Y-%m-%d %H:%M:%S",
        ))
        .or(Utc.datetime_from_str(
            format!("{}-01 00:00:00", input).as_str(),
            "%Y-%m-%d %H:%M:%S",
        ))
        .or(Utc.datetime_from_str(
            format!("{}-01-01 00:00:00", input).as_str(),
            "%Y-%m-%d %H:%M:%S",
        ))
}

pub fn filter_projects_by_timestamps(projects: Projects, filter_before: &Option<DateTime<Utc>>, filter_after: &Option<DateTime<Utc>>) -> Projects {
    let check_timestamp = |time_stamp: &DateTime<Utc>, filter: &Option<DateTime<Utc>>, after: bool| {
        if let &Some(unwraped) = filter {
            if after {
                time_stamp >= &unwraped
            } else {
                time_stamp <= &unwraped
            }
        } else {
            debug!("helper::filter_projects_by_timestamps::check_timestamp:: is none");
            true
        }
    };

    let mut out = Projects::default();
    for mut project in projects {
        project.notes = project
            .notes
            .into_iter()
            .filter(|note| {
                check_timestamp(&note.time_stamp, filter_before, false)
            })
            .filter(|note| check_timestamp(&note.time_stamp, filter_after, true))
            .collect();

        if project.notes.len() != 0 {
            out.insert(project);
        }
    }

    out
}
