extern crate chrono;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate serde;

use chrono::prelude::*;
use std::collections::BTreeSet as DataSet;

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
    fn archive_project(&self, &ProjectName) -> Result<(), errors::Error>;
    fn get_notes(&self, &ProjectName) -> Result<Notes, errors::Error>;
    fn get_project(&self, &ProjectName) -> Result<Project, errors::Error>;
    fn get_projects(&self) -> Result<Projects, errors::Error>;
    fn write_note(&self, &ProjectName, &Note) -> Result<(), errors::Error>;
}

#[derive(Debug, Serialize,Deserialize,Ord,PartialOrd,Eq,PartialEq)]
pub struct Note {
    pub time_stamp: DateTime<Utc>,
    pub value: String,
}

pub type Notes = DataSet<Note>;

pub struct ProjectName(String);

const PROJECT_SEPPERATOR: &'static str = ".";

impl ProjectName {
    pub fn normalize_path(&self) -> String {
        self.0.replace(PROJECT_SEPPERATOR, "/")
    }

    pub fn new(name: String) -> ProjectName {
        ProjectName(name)
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

pub struct Project {
    pub name: ProjectName,
    pub archived: bool,
    pub notes: Notes,
}

pub type Projects = DataSet<Project>;
