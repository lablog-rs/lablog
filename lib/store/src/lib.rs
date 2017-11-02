extern crate chrono;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate serde;

use chrono::prelude::*;
use std::collections::BTreeSet;

pub mod errors {
    error_chain!{
        errors {
          ProjectAlreadyArchived(project: String) {
            description("project is already archived")
            display("project '{}' is already archived", project)
          }
          NoProjectWithThisName(project: String) {
            description("no project with this name")
            display("no project with the name '{}' found", project)
          }
          NoteHasEmptyValue {
            description("not has empty value")
            display("note has empty value")
          }
        }
    }
}

pub trait Store {
    /// Archive the given project.
    fn archive_project(&self, &ProjectName) -> Result<(), errors::Error>;
    /// Gets a single project with its notes from the store. If archived is
    /// true the store will try
    /// to fetch the project from the archive.
    fn get_project(&self, ProjectName, bool) -> Result<Project, errors::Error>;
    /// Gets all projects and their notes from the store.
    fn get_projects(&self) -> Result<Projects, errors::Error>;
    /// Gets a list of the names of all projects from the store.
    fn get_projects_list(&self) -> Result<BTreeSet<ProjectName>, errors::Error>;
    /// Write a note for a project to the store.
    fn write_note(&self, &ProjectName, &Note) -> Result<(), errors::Error>;
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Note {
    pub time_stamp: DateTime<Utc>,
    pub value: String,
}

impl From<String> for Note {
    fn from(input: String) -> Self {
        Note {
            value: input,
            time_stamp: Utc::now(),
        }
    }
}

pub type Notes = BTreeSet<Note>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ProjectName(String);

impl std::fmt::Display for ProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> std::convert::Into<&'a str> for &'a ProjectName {
    fn into(self) -> &'a str {
        (self.0).as_str()
    }
}

pub const PROJECT_SEPPERATOR: &str = ".";

impl ProjectName {
    /// generates a unix path out of the project name
    pub fn normalize_path(&self) -> String {
        self.0.replace(PROJECT_SEPPERATOR, "/")
    }
}

impl From<String> for ProjectName {
    fn from(string: String) -> Self {
        ProjectName(string)
    }
}

impl<'a> From<&'a str> for ProjectName {
    fn from(string: &'a str) -> Self {
        ProjectName(string.into())
    }
}

impl std::str::FromStr for ProjectName {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(string.into())
    }
}

#[cfg(test)]
mod test_lablog_store_projectname {
    use ProjectName;

    #[test]
    fn normalize_path() {
        let expected = "/test/test/test";
        let got = ProjectName(".test.test.test".to_string()).normalize_path();

        assert_eq!(expected, got)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Project {
    pub name: ProjectName,
    pub archived: bool,
    pub notes: Notes,
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub type Projects = BTreeSet<Project>;
