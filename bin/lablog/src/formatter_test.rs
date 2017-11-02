use chrono::TimeZone;
use chrono::Utc;
use formatter::*;
use store::note::Note;
use store::project_name::ProjectName;

#[test]
fn test_format_project_name() {
    let expected = String::from("== Test");
    let input: ProjectName = "Test".into();
    let got = format_project_name(&input);

    assert_eq!(expected, got);
}

#[test]
fn test_format_note() {
    let expected = String::from("=== 1970-01-01 00:00:00 UTC\ntest");
    let input = Note {
        time_stamp: Utc.timestamp(0, 0),
        value: String::from("test"),
    };

    let got = format_note(&input);

    assert_eq!(expected, got);
}
