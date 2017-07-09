#[macro_use]
extern crate log;

extern crate csv;
extern crate lablog_store as store;
extern crate walkdir;

use errors::ResultExt;
use std::collections::BTreeSet as DataSet;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use store::*;
use walkdir::WalkDir;

const FILE_EXTENTION: &'static str = "csv";

pub struct CSVStore {
    pub data_dir: PathBuf,
}

impl CSVStore {
    #[allow(dead_code)]
    fn new(data_dir: PathBuf) -> CSVStore {
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

    fn project_name_from_path(&self, path: PathBuf) -> Result<ProjectName, errors::Error> {
        let path = path.strip_prefix(&self.data_dir)
            .expect("path should always have the data_dir as an prefix")
            .with_extension("");

        let name = path.to_str()
            .expect("lets hope that the path is valid utf8")
            .replace("/", store::PROJECT_SEPPERATOR);

        Ok(ProjectName::new(name))
    }
}

impl store::Store for CSVStore {
    #[allow(unused_variables)]
    fn archive_project(&self, name: &ProjectName) -> Result<(), errors::Error> {
        unimplemented!();
    }

    #[allow(unused_variables)]
    fn get_notes(&self, name: &ProjectName) -> Result<Notes, errors::Error> {
        let filepath = self.project_path(name);

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(filepath)
            .chain_err(|| "can not build reader to read note from file")?;

        let mut notes = Notes::new();
        for result in rdr.deserialize() {
            let record: Note = result.chain_err(|| "can not deserialize note")?;
            notes.insert(record);
        }

        Ok(notes)
    }

    #[allow(unused_variables)]
    fn get_project(&self, name: &ProjectName) -> Result<Project, errors::Error> {
        unimplemented!();
    }

    fn get_projects(&self) -> Result<Projects, errors::Error> {
        unimplemented!();
    }

    fn get_projects_list(&self) -> Result<DataSet<ProjectName>, errors::Error> {
        let mut list = DataSet::new();

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
    use std::collections::BTreeSet as DataSet;
    use store::*;

    #[test]
    fn write_read_note() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note")
            .expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testproject = ProjectName::new("test".to_string());

        let mut notes = Notes::new();

        for i in 0..100 {
            let note = Note {
                time_stamp: Utc::now(),
                value: "test".to_string() + &i.to_string(),
            };

            teststore.write_note(&testproject, &note).expect(
                "can not write note to store",
            );

            notes.insert(note);
        }

        let storenotes = teststore.get_notes(&testproject).expect(
            "can not get note from store",
        );

        println!("{:#?}", storenotes);

        if notes.len() != storenotes.len() {
            panic!(
                "storenotes length ({}) is different from notes length ({})",
                storenotes.len(),
                notes.len()
            )
        }

        assert_eq!(notes, storenotes);
    }

    #[test]
    #[should_panic]
    fn write_empty_note() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note")
            .expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testproject = ProjectName::new("test".to_string());

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

        let mut list = DataSet::new();

        for i in 1..100 {
            let testproject = ProjectName::new("test".to_string() + &i.to_string());

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
            let expected = ProjectName::new("test".to_string());
            let path = teststore.project_path(&expected);

            let got = teststore.project_name_from_path(path).expect(
                "can not get project name from path",
            );

            assert_eq!(expected, got);
        }

        {
            let expected = ProjectName::new("test.test".to_string());
            let path = teststore.project_path(&expected);

            let got = teststore.project_name_from_path(path).expect(
                "can not get project name from path",
            );

            assert_eq!(expected, got);
        }

        {
            let expected = ProjectName::new("test.test.test".to_string());
            let path = teststore.project_path(&expected);

            let got = teststore.project_name_from_path(path).expect(
                "can not get project name from path",
            );

            assert_eq!(expected, got);
        }
    }
}
