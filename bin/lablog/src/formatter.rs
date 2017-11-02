use std::io::Write;
use store::Note;
use store::ProjectName;

pub fn format_project_name<T: Write>(writer: &mut T, project_name: &ProjectName) {
    writeln!(writer, "== {}", project_name).unwrap()
}

pub fn format_note<T: Write>(writer: &mut T, note: &Note) {
    writeln!(writer, "=== {}\n{}", note.time_stamp, note.value).unwrap()
}
