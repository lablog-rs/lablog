extern crate chrono;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate serde;

use chrono::prelude::*;
use std::collections::BTreeSet as DataSet;

pub mod lablog_store_errors {
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
        }
    }
}

use lablog_store_errors as errors;

pub trait Store {
    fn archive_project(&self, &ProjectName) -> Result<(), errors::Error>;
    fn get_notes(&self, &ProjectName) -> Result<Notes, errors::Error>;
    fn get_project(&self, &ProjectName) -> Result<Project, errors::Error>;
    fn get_projects(&self) -> Result<Projects, errors::Error>;
    fn write_note(&self, &ProjectName, &Note) -> Result<(), errors::Error>;
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Note {
    pub time_stamp: DateTime<UTC>,
    pub value: String,
}

pub type Notes = DataSet<Note>;

pub type ProjectName = String;

pub struct Project {
    pub name: ProjectName,
    pub archived: bool,
    pub notes: Notes,
}

pub type Projects = DataSet<Project>;
