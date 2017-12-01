use std::collections::{
    BTreeMap,
    BTreeSet,
};
use store::note::Note;
use store::project::Projects;
use store::project_name::ProjectName;

pub trait Formatter {
    fn project_name(&self, &ProjectName) -> String;
    fn note(&self, &Note) -> String;
    fn projects(&self, &Projects) -> String;
    fn search_results(&self, &BTreeMap<ProjectName, BTreeSet<String>>) -> String;
}

pub mod asciidoc {
    use ansi_term::Style;
    use formatter::Formatter;
    use std::collections::{
        BTreeMap,
        BTreeSet,
    };
    use store::note::Note;
    use store::project::Projects;
    use store::project_name::ProjectName;

    #[derive(Debug, Default)]
    pub struct FormatterAsciidoc {}

    impl Formatter for FormatterAsciidoc {
        fn project_name(&self, project_name: &ProjectName) -> String {
            format!(
                "{}",
                Style::new().bold().paint(format!("== {}", project_name))
            )
        }

        fn note(&self, note: &Note) -> String {
            format!("=== {}\n{}", note.time_stamp, note.value)
        }

        fn projects(&self, projects: &Projects) -> String {
            let mut out = String::new();

            for project in projects {
                if project.archived {
                    continue;
                }

                out = format!(
                    "{}{project_name}\n",
                    out,
                    project_name = self.project_name(&project.name)
                );

                for note in &project.notes {
                    out = format!("{}{note}\n", out, note = self.note(note));
                }
            }

            out.trim().into()
        }

        fn search_results(&self, results: &BTreeMap<ProjectName, BTreeSet<String>>) -> String {
            let mut out = String::new();

            for (project, matches) in results {
                out.push_str(format!("{}\n", self.project_name(project)).as_str());

                for entry in matches {
                    out.push_str(format!("{}\n", entry).as_str());
                }

                out.push_str("\n");
            }

            out.trim().into()
        }
    }

    #[cfg(test)]
    mod test {
        use Formatter;
        use FormatterAsciidoc;
        use chrono::TimeZone;
        use chrono::Utc;
        use store::note::Note;
        use store::project::Project;
        use store::project::Projects;
        use store::project_name::ProjectName;

        #[test]
        fn test_format_project_name_default() {
            let formatter = FormatterAsciidoc::default();
            let expected = String::from("== ");
            let input = ProjectName::default();
            let got = formatter.project_name(&input);

            assert_eq!(expected, got);
        }

        #[test]
        fn test_format_note_default() {
            let formatter = FormatterAsciidoc::default();
            let expected = String::from("=== 1970-01-01 00:00:00 UTC\n");
            let input = Note {
                time_stamp: Utc.timestamp(0, 0),
                ..Note::default()
            };

            let got = formatter.note(&input);

            assert_eq!(expected, got);
        }

        #[test]
        fn test_projects_default() {
            let formatter = FormatterAsciidoc::default();
            let expected = String::from("");
            let input = Projects::default();
            let got = formatter.projects(&input);

            assert_eq!(expected, got);
        }

        #[test]
        fn test_projects_single_default() {
            let formatter = FormatterAsciidoc::default();
            let expected = String::from("==");
            let input = {
                let mut input = Projects::default();
                input.insert(Project::default());

                input
            };

            let got = formatter.projects(&input);

            assert_eq!(expected, got);
        }

        #[test]
        fn test_projects_single_content() {
            let formatter = FormatterAsciidoc::default();
            let expected = String::from("== Test1\n=== 1970-01-01 00:00:00 UTC\nTestNote1");
            let input = {
                let mut input = Projects::default();
                {
                    let mut test1 = Project {
                        name: "Test1".into(),
                        ..Project::default()
                    };
                    test1.notes.insert(Note {
                        value: "TestNote1".into(),
                        time_stamp: Utc.timestamp(0, 0),
                    });
                    input.insert(test1);
                }

                input
            };

            let got = formatter.projects(&input);

            assert_eq!(expected, got);
        }

        #[test]
        fn test_projects_multiple_content() {
            let formatter = FormatterAsciidoc::default();
            let expected = String::from("== Test1\n=== 1970-01-01 00:00:00 UTC\nTestNote1\n== Test2\n=== 1970-01-01 00:00:00 UTC\nTestNote1");

            let input = {
                let mut input = Projects::default();
                {
                    let mut project = Project {
                        name: "Test1".into(),
                        ..Project::default()
                    };
                    project.notes.insert(Note {
                        value: "TestNote1".into(),
                        time_stamp: Utc.timestamp(0, 0),
                    });
                    input.insert(project);
                }

                {
                    let mut project = Project {
                        name: "Test2".into(),
                        ..Project::default()
                    };
                    project.notes.insert(Note {
                        value: "TestNote1".into(),
                        time_stamp: Utc.timestamp(0, 0),
                    });
                    input.insert(project);
                }

                input
            };

            let got = formatter.projects(&input);

            assert_eq!(expected, got);
        }
    }
}
