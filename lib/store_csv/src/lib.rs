extern crate lablog_store as store;

use std::path::PathBuf;
use store::*;
use store::lablog_store_errors as errors;

pub struct CSVStore {
    pub data_dir: PathBuf,
}

impl CSVStore {
    fn new(data_dir: PathBuf) -> CSVStore {
        CSVStore { data_dir: data_dir }
    }
}

impl store::Store for CSVStore {
    fn archive_project(&self, name: &ProjectName) -> Result<(), errors::Error> {
        unimplemented!();
    }

    fn get_notes(&self, name: &ProjectName) -> Result<Notes, errors::Error> {
        unimplemented!();
    }

    fn get_project(&self, name: &ProjectName) -> Result<Project, errors::Error> {
        unimplemented!();
    }

    fn get_projects(&self) -> Result<Projects, errors::Error> {
        unimplemented!();
    }

    fn write_note(&self, name: &ProjectName, note: &Note) -> Result<(), errors::Error> {
        unimplemented!();
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
        let data_dir = TempDir::new("lablog_store_csv_test_write_read_note").expect("can not create temporary directory for test");
        let teststore = CSVStore::new(data_dir.path().to_path_buf());
        let testproject = "test".to_string();

        let note = Note {
            time_stamp: UTC::now(),
            value: "test".to_string(),
        };

        let result = teststore
            .write_note(&testproject, &note)
            .expect("can not write note to store");

        let notes = teststore.get_notes(&testproject);
    }
}
