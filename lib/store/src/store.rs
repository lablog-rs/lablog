use errors;
use note::Note;
use project::Project;
use project::Projects;
use project_name::ProjectName;
use std::collections::BTreeSet;

pub trait Store {
    /// Archive the given project.
    fn archive_project(&self, &ProjectName) -> Result<(), errors::Error>;
    /// Gets a single project with its notes from the store. If archived is
    /// true the store will try to fetch the project from the archive.
    fn get_project(&self, ProjectName, bool) -> Result<Project, errors::Error>;
    /// Gets all projects and their notes from the store.
    fn get_projects(&self) -> Result<Projects, errors::Error>;
    /// Gets a list of the names of all projects from the store.
    fn get_projects_list(&self) -> Result<BTreeSet<ProjectName>, errors::Error>;
    /// Write a note for a project to the store.
    fn write_note(&self, &ProjectName, &Note) -> Result<(), errors::Error>;
}
