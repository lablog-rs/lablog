#[macro_use]
extern crate log;

extern crate csv;
extern crate lablog_store as store;
extern crate walkdir;

use errors::ResultExt;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use store::*;
use walkdir::WalkDir;

const FILE_EXTENTION: &'static str = "csv";

pub struct CSVStore {
    pub data_dir: PathBuf,
}

impl CSVStore {
    #[allow(dead_code)]
    pub fn new(data_dir: PathBuf) -> CSVStore {
        CSVStore { data_dir: data_dir }
    }

    fn project_path(&self, name: &ProjectName) -> PathBuf {
        let project_path = name.normalize_path();

        // clone store data_dir path and append project_path.FILE_EXTENTION
        let mut path = self.data_dir.clone();
        path.push(project_path);
        path.set_extension(FILE_EXTENTION);

        path
    }

    fn project_archive_path(&self, name: &ProjectName) -> PathBuf {
        let project_path = name.normalize_path();

        // clone store data_dir path and append project_path.FILE_EXTENTION
        let mut path = self.data_dir.clone();
        path.push(".archive");
        path.push(project_path);
        path.set_extension(FILE_EXTENTION);

        path
    }

    fn project_name_from_path(&self, path: PathBuf) -> Result<ProjectName, errors::Error> {
        let path = path.strip_prefix(&self.data_dir)
            .expect("path should always have the data_dir as an prefix")
            .with_extension("");

        let name = path.to_str()
            .expect("lets hope that the path is valid utf8")
            .replace("/", store::PROJECT_SEPPERATOR);

        Ok(name.into())
    }
}

impl store::Store for CSVStore {
    fn archive_project(&self, name: &ProjectName) -> Result<(), errors::Error> {
        let oldpath = self.project_path(name);
        let newpath = self.project_archive_path(name);

        println!("newpath: {:#?}", newpath);

        // either append existing notes or move the old file
        if newpath.exists() {
            {
                let mut newfile = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(newpath)
                    .chain_err(|| "can not open archive file for appending")?;

                let mut oldfile = File::open(&oldpath).chain_err(
                    || "can not open original file for reading",
                )?;

                let mut buffer = Vec::new();
                oldfile.read_to_end(&mut buffer).chain_err(
                    || "can not read original files content",
                )?;

                newfile.write_all(&buffer).chain_err(
                    || "can not append original content to new file",
                )?;
            }

            fs::remove_file(oldpath).chain_err(
                || "can not remove original file",
            )?;
        } else {
            fs::create_dir_all(newpath.parent().expect(
                "archive path is root path? this seems very wrong",
            )).chain_err(|| "can not create directory for archive")?;

            fs::rename(oldpath, newpath).chain_err(
                || "can not move project file to archive path",
            )?;
        }

        Ok(())
    }

    fn get_project(&self, name: ProjectName, archived: bool) -> Result<Project, errors::Error> {
        let filepath = {
            if archived {
                self.project_archive_path(&name)
            } else {
                self.project_path(&name)
            }
        };

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(filepath)
            .chain_err(|| "can not build reader to read note from file")?;

        let mut notes = Notes::new();
        for result in rdr.deserialize() {
            let record: Note = result.chain_err(|| "can not deserialize note")?;
            notes.insert(record);
        }

        Ok(Project {
            archived: archived,
            name: name,
            notes: notes,
        })
    }

    fn get_projects(&self) -> Result<Projects, errors::Error> {
        let list = self.get_projects_list().chain_err(
            || "can not get projects list",
        )?;

        let mut projects = Projects::new();
        for item in list {
            let project = self.get_project(item, false).chain_err(
                || "can not get project",
            )?;
            projects.insert(project);
        }

        Ok(projects)
    }

    fn get_projects_list(&self) -> Result<BTreeSet<ProjectName>, errors::Error> {
        let mut list = BTreeSet::new();

        for entry in WalkDir::new(&self.data_dir) {
            let entry = entry.chain_err(|| "can not get entry from walkdir")?;

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let name = self.project_name_from_path(path.to_path_buf()).chain_err(
                || "can not get project name from entry path",
            )?;

            list.insert(name);
        }

        Ok(list)
    }

    fn write_note(&self, name: &ProjectName, note: &Note) -> Result<(), errors::Error> {
        if note.value == "" {
            return Err(store::errors::ErrorKind::NoteHasEmptyValue.into());
        }

        let file = {
            let filepath = self.project_path(name);
            trace!("write_note: filepath: {:#?}", filepath);

            fs::create_dir_all(filepath.parent().expect(
                "filepath is root path? this seems very wrong",
            )).chain_err(|| "can not create directory for file")?;

            OpenOptions::new()
                .append(true)
                .create(true)
                .open(filepath)
                .chain_err(|| "can not open project file")?
        };

        let mut wtr = csv::Writer::from_writer(file);

        // we dont want to have headers in our files so we use the tuple pattern to
        // avoid that
        wtr.serialize((&note.time_stamp, &note.value)).chain_err(
            || "can not serialize note to csv",
        )?;

        wtr.flush().chain_err(|| "can not flush csv writer")?;

        Ok(())
    }
}

#[cfg(test)]
mod test_lablog_store_csv {
    extern crate tempdir;
    extern crate chrono;
    extern crate lablog_store as store;

    use self::chrono::prelude::*;
    use self::tempdir::TempDir;
    use CSVStore;
    use std::collections::BTreeSet;
    use store::*;

    #[test]
    fn write_read_notes() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note")
            .expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testname = "test".into();

        let mut notes = Notes::new();

        for i in 0..100 {
            let note = Note {
                time_stamp: Utc::now(),
                value: "test".to_string() + &i.to_string(),
            };

            teststore.write_note(&testname, &note).expect(
                "can not write note to store",
            );

            notes.insert(note);
        }

        let testproject = Project {
            archived: false,
            name: testname.clone(),
            notes: notes,
        };

        let storeproject = teststore.get_project(testname, false).expect(
            "can not get note from store",
        );

        println!("storeproject: {:#?}", storeproject);

        if testproject.notes.len() != storeproject.notes.len() {
            panic!(
                "storenotes length ({}) is different from notes length ({})",
                testproject.notes.len(),
                storeproject.notes.len()
            )
        }

        assert_eq!(testproject, storeproject);
    }

    #[test]
    #[should_panic]
    fn write_empty_note() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note")
            .expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testproject = "test".into();

        let note = Note {
            time_stamp: Utc::now(),
            value: "".to_string(),
        };

        teststore.write_note(&testproject, &note).expect(
            "can not write note to store",
        );
    }

    #[test]
    fn get_projects_list() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note")
            .expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());

        let note = Note {
            time_stamp: Utc::now(),
            value: "test".to_string(),
        };

        let mut list = BTreeSet::new();

        for i in 1..100 {
            let testproject = format!("test{}", i).into();

            teststore.write_note(&testproject, &note).expect(
                "can not write note to store",
            );

            list.insert(testproject);
        }

        let storelist = teststore.get_projects_list().expect(
            "can not get project list from store",
        );

        println!("storelist: {:#?}", storelist);

        if list.len() != storelist.len() {
            panic!(
                "storelist length ({}) is not list length ({})",
                storelist.len(),
                list.len()
            )
        }

        assert_eq!(list, storelist);
    }

    #[test]
    fn project_name_from_path() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note")
            .expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        {
            let expected = "test".into();
            let path = teststore.project_path(&expected);

            let got = teststore.project_name_from_path(path).expect(
                "can not get project name from path",
            );

            assert_eq!(expected, got);
        }

        {
            let expected = "test.test".into();
            let path = teststore.project_path(&expected);

            let got = teststore.project_name_from_path(path).expect(
                "can not get project name from path",
            );

            assert_eq!(expected, got);
        }

        {
            let expected = "test.test.test".into();
            let path = teststore.project_path(&expected);

            let got = teststore.project_name_from_path(path).expect(
                "can not get project name from path",
            );

            assert_eq!(expected, got);
        }
    }

    #[test]
    fn get_projects() {
        let data_dir = TempDir::new("lablog_store_csv_test_get_projects").expect(
            "can not create temporary directory for test",
        );
        let teststore = CSVStore::new(data_dir.path().to_path_buf());

        let note = Note {
            time_stamp: Utc::now(),
            value: "test".to_string(),
        };

        let mut notes = Notes::new();
        notes.insert(note.clone());

        let mut testprojects = Projects::new();

        for i in 1..100 {
            let testproject = format!("test{}", i).into();

            teststore.write_note(&testproject, &note).expect(
                "can not write note to store",
            );

            testprojects.insert(Project {
                name: testproject,
                notes: notes.clone(),
                archived: false,
            });
        }

        let storeprojects = teststore.get_projects().expect(
            "can not get projects from store",
        );

        println!("storeprojects: {:#?}", storeprojects);

        if testprojects.len() != storeprojects.len() {
            panic!(
                "storeprojects length ({}) is not testprojects length ({})",
                storeprojects.len(),
                testprojects.len()
            )
        }

        assert_eq!(testprojects, storeprojects);
    }

    #[test]
    fn archive_project() {
        let data_dir = TempDir::new("lablog_store_csv_test_get_projects").expect(
            "can not create temporary directory for test",
        );
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testname = "test".into();

        let note = Note {
            time_stamp: Utc::now(),
            value: "test".to_string(),
        };

        teststore.write_note(&testname, &note).expect(
            "can not write note to store",
        );

        let mut notes = Notes::new();
        notes.insert(note);

        teststore.archive_project(&testname).expect(
            "can not archive project",
        );

        let testproject = Project {
            archived: true,
            name: testname.clone(),
            notes: notes,
        };

        let storeproject = teststore.get_project(testname, true).expect(
            "can not get project from store",
        );

        assert_eq!(testproject, storeproject);
    }

    #[test]
    fn archive_project_merging() {
        let data_dir = TempDir::new("lablog_store_csv_test_get_projects").expect(
            "can not create temporary directory for test",
        );
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testname = "test".into();

        let note = Note {
            time_stamp: Utc::now(),
            value: "test".to_string(),
        };

        let note2 = Note {
            time_stamp: Utc::now(),
            value: "test2".to_string(),
        };

        teststore.write_note(&testname, &note).expect(
            "can not write note to store",
        );

        teststore.archive_project(&testname).expect(
            "can not archive project",
        );

        teststore.write_note(&testname, &note2).expect(
            "can not write note to store",
        );

        teststore.archive_project(&testname).expect(
            "can not archive project",
        );

        let mut notes = Notes::new();
        notes.insert(note);
        notes.insert(note2);

        let testproject = Project {
            archived: true,
            name: testname.clone(),
            notes: notes,
        };

        let storeproject = teststore.get_project(testname, true).expect(
            "can not get project from store",
        );

        assert_eq!(testproject, storeproject);
    }
}
