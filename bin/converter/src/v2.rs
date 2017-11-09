extern crate lablog_lib_v2 as v2;

use self::v2::{
    get_projects,
    get_projects_notes,
    ProjectsNotes,
};
use std::path::PathBuf;

pub fn read_notes(datadir: &PathBuf) -> ProjectsNotes {
    let projects = get_projects(datadir, Some("".into()));

    trace!("v2: notes: projects: {:#?}", projects);

    get_projects_notes(datadir, projects)
}
