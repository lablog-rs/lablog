use store::note::Note;
use store::project_name::ProjectName;

pub fn format_project_name(project_name: &ProjectName) -> String {
    format!("== {}", project_name)
}

pub fn format_note(note: &Note) -> String {
    format!("=== {}\n{}", note.time_stamp, note.value)
}
