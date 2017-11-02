use std::io::Error;
use std::io::Write;
use std::result::Result;
use store::Note;
use store::ProjectName;

pub fn format_project_name<T: Write>(
    writer: &mut T,
    project_name: &ProjectName,
) -> Result<(), Error> {
    writeln!(writer, "== {}", project_name)
}

pub fn format_note<T: Write>(writer: &mut T, note: &Note) -> Result<(), Error> {
    writeln!(writer, "=== {}\n{}", note.time_stamp, note.value)
}
