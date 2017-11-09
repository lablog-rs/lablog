use note::Notes;
use project_name::ProjectName;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Project {
    pub name: ProjectName,
    pub archived: bool,
    pub notes: Notes,
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)
    }
}

pub type Projects = BTreeSet<Project>;
