use chrono::{
    DateTime,
    Duration,
    Local,
    ParseResult,
    TimeZone,
    Utc,
};
use clap::ArgMatches;
use errors::*;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use store::project::Projects;
use store::store::Store;
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
        .or_else(|_| {
            Utc.datetime_from_str(input.as_str(), "%Y-%m-%d %H:%M:%S")
        })
        .or_else(|_| {
            Utc.datetime_from_str(format!("{}:00", input).as_str(), "%Y-%m-%d %H:%M:%S")
        })
        .or_else(|_| {
            Utc.datetime_from_str(format!("{}:00:00", input).as_str(), "%Y-%m-%d %H:%M:%S")
        })
        .or_else(|_| {
            Utc.datetime_from_str(format!("{} 00:00:00", input).as_str(), "%Y-%m-%d %H:%M:%S")
        })
        .or_else(|_| {
            Utc.datetime_from_str(
                format!("{}-01 00:00:00", input).as_str(),
                "%Y-%m-%d %H:%M:%S",
            )
        })
        .or_else(|_| {
            Utc.datetime_from_str(
                format!("{}-01-01 00:00:00", input).as_str(),
                "%Y-%m-%d %H:%M:%S",
            )
        })
}

pub fn filter_projects_by_timestamps(projects: Projects, filter_before: &Option<DateTime<Utc>>, filter_after: &Option<DateTime<Utc>>) -> Projects {
    let check_timestamp = |time_stamp: &DateTime<Utc>, filter: &Option<DateTime<Utc>>, after: bool| {
        if let Some(unwraped) = *filter {
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

        if !project.notes.is_empty() {
            out.insert(project);
        }
    }

    out
}

pub struct Filters {
    project_name: Option<Regex>,
    timestamp_before: Option<DateTime<Utc>>,
    timestamp_after: Option<DateTime<Utc>>,
}

pub fn get_filtered_projects(store: &Store, filters: &Filters) -> Result<Projects> {
    let mut projects = store
        .get_projects()
        .chain_err(|| "can not get projects from store")?;

    if let Some(ref filter) = filters.project_name {
        projects = projects
            .into_iter()
            .filter(|project| filter.is_match((&project.name).into()))
            .collect();
    }

    Ok(filter_projects_by_timestamps(
        projects,
        &filters.timestamp_before,
        &filters.timestamp_after,
    ))
}

pub fn get_filters_from_match(matches: &ArgMatches) -> Result<Filters> {
    let project_name = {
        let arg = matches.value_of("filter_project_name");

        if arg.is_some() {
            Some(Regex::new(arg.unwrap())
                .chain_err(|| "can not create regex out of filter argument")?)
        } else {
            None
        }
    };

    let timestamp_before = {
        let arg = matches.value_of("filter_before");

        if arg.is_none() {
            None
        } else {
            let timestamp = try_multiple_time_parser(arg.unwrap()).chain_err(|| "can not parse before filter timestamp")?;
            Some(timestamp)
        }
    };

    let timestamp_after = {
        let arg = matches.value_of("filter_after");

        if arg.is_none() {
            None
        } else {
            let timestamp = try_multiple_time_parser(arg.unwrap()).chain_err(|| "can not parse before filter timestamp")?;
            Some(timestamp)
        }
    };

    Ok(Filters {
        project_name: project_name,
        timestamp_before: timestamp_before,
        timestamp_after: timestamp_after,
    })
}

#[cfg(test)]
mod test {
    use chrono::{
        Duration,
        Utc,
    };
    use helper::try_multiple_time_parser;
    use std::collections::BTreeSet;
    use store::errors;
    use store::note::Note;
    use store::project::Project;
    use store::project::Projects;
    use store::project_name::ProjectName;
    use store::store::Store;

    #[derive(Default)]
    struct TestStore {}

    impl Store for TestStore {
        fn archive_project(&self, _: &ProjectName) -> Result<(), errors::Error> {
            unimplemented!()
        }
        fn get_project(&self, _: ProjectName, _: bool) -> Result<Project, errors::Error> {
            unimplemented!()
        }
        fn get_projects(&self) -> Result<Projects, errors::Error> {
            unimplemented!()
        }
        fn get_projects_list(&self) -> Result<BTreeSet<ProjectName>, errors::Error> {
            unimplemented!()
        }
        fn write_note(&self, _: &ProjectName, _: &Note) -> Result<(), errors::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn test_try_multiple_time_parser_today() {
        let input = "today";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = Utc::now().date().and_hms(0, 0, 0).to_rfc3339();

        println!("expected: {}", expected);

        assert_eq!(output.to_rfc3339(), expected)
    }

    #[test]
    fn test_try_multiple_time_parser_yesterday() {
        let input = "yesterday";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = Utc::now()
            .date()
            .and_hms(0, 0, 0)
            .checked_sub_signed(Duration::days(1))
            .unwrap()
            .to_rfc3339();

        println!("expected: {}", expected);

        assert_eq!(output.to_rfc3339(), expected)
    }

    #[test]
    fn test_try_multiple_time_parser_year() {
        let input = "2001";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = "2001-01-01T00:00:00+00:00";

        assert_eq!(output.to_rfc3339(), expected)
    }

    #[test]
    fn test_try_multiple_time_parser_year_month() {
        let input = "2001-02";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = "2001-02-01T00:00:00+00:00";

        assert_eq!(output.to_rfc3339(), expected)
    }

    #[test]
    fn test_try_multiple_time_parser_year_month_day() {
        let input = "2001-02-03";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = "2001-02-03T00:00:00+00:00";

        assert_eq!(output.to_rfc3339(), expected)
    }

    #[test]
    fn test_try_multiple_time_parser_year_month_day_hour() {
        let input = "2001-02-03 04";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = "2001-02-03T04:00:00+00:00";

        assert_eq!(output.to_rfc3339(), expected)
    }

    #[test]
    fn test_try_multiple_time_parser_year_month_day_hour_minute() {
        let input = "2001-02-03 04:05";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = "2001-02-03T04:05:00+00:00";

        assert_eq!(output.to_rfc3339(), expected)
    }

    #[test]
    fn test_try_multiple_time_parser_year_month_day_hour_minute_seconds() {
        let input = "2001-02-03 04:05:06";
        let output = try_multiple_time_parser(input).unwrap();
        let expected = "2001-02-03T04:05:06+00:00";

        assert_eq!(output.to_rfc3339(), expected)
    }
}
