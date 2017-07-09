#[macro_use]
extern crate log;

extern crate csv;
extern crate lablog_store as store;

use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use store::*;
use store::errors::ResultExt;

const FILE_EXTENTION: &'static str = "csv";

pub struct CSVStore {
    pub data_dir: PathBuf,
}

impl CSVStore {
    #[allow(dead_code)]
    fn new(data_dir: PathBuf) -> CSVStore {
        CSVStore { data_dir: data_dir }
    }

    fn project_filepath(&self, name: &ProjectName) -> PathBuf {
        let project_path = name.normalize_path();

        // clone store data_dir path and append project_path.FILE_EXTENTION
        let mut path = self.data_dir.clone();
        path.push(project_path);
        path.set_extension(FILE_EXTENTION);

        path
    }
}

impl store::Store for CSVStore {
    #[allow(unused_variables)]
    fn archive_project(&self, name: &ProjectName) -> Result<(), errors::Error> {
        unimplemented!();
    }

    #[allow(unused_variables)]
    fn get_notes(&self, name: &ProjectName) -> Result<Notes, errors::Error> {
        let filepath = self.project_filepath(name);

        let mut rdr = csv::ReaderBuilder::new().has_headers(false)
            .from_path(filepath)
            .chain_err(|| "can not build reader to read note from file")?;

        let mut iter = rdr.deserialize();
        let mut notes = Notes::default();

        if let Some(result) = iter.next() {
            let record: Note = result.chain_err(|| "can not deserialize note")?;
            notes.insert(record);
        } else {
            return Err(From::from("expected at least one record but got none"));
        };

        Ok(notes)
    }

    #[allow(unused_variables)]
    fn get_project(&self, name: &ProjectName) -> Result<Project, errors::Error> {
        unimplemented!();
    }

    fn get_projects(&self) -> Result<Projects, errors::Error> {
        unimplemented!();
    }

    fn write_note(&self, name: &ProjectName, note: &Note) -> Result<(), errors::Error> {
        if note.value == "" {
            return Err(store::errors::ErrorKind::NoteHasEmptyValue.into());
        }

        let file = {
            let filepath = self.project_filepath(name);
            trace!("write_note: filepath: {:#?}", filepath);

            fs::create_dir_all(filepath
                               .parent()
                               .expect("filepath is root path? this seems very wrong")).chain_err(|| "can not create directory for file")?;

            OpenOptions::new().append(true)
                .create(true)
                .open(filepath)
                .chain_err(|| "can not open project file")?
        };

        let mut wtr = csv::Writer::from_writer(file);

        // we dont want to have headers in our files so we use the tuple pattern to
        // avoid that
        wtr.serialize((&note.time_stamp, &note.value))
            .chain_err(|| "can not serialize note to csv")?;

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
    use store::*;

    #[test]
    fn write_read_note() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note").expect("can not create
        temporary directory for test");

        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testproject = ProjectName::new("test".to_string());

        let note = Note {
            time_stamp: Utc::now(),
            value: "test".to_string(),
        };

        teststore
            .write_note(&testproject, &note)
            .expect("can not write note to store");

        let notes = teststore
            .get_notes(&testproject)
            .expect("can not get note from store");

        if notes.len() != 1 {
            panic!("did not get enough notes back from store")
        }

        if !notes.contains(&note) {
            panic!("did not get the test note back from store")
        }
    }

    #[test]
    #[should_panic]
    fn write_empty_note() {
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note").expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testproject = ProjectName::new("test".to_string());

        let note = Note {
            time_stamp: Utc::now(),
            value: "".to_string(),
        };

        teststore
            .write_note(&testproject, &note)
            .expect("can not write note to store");
    }
}
