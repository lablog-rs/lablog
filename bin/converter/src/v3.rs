extern crate chrono;
extern crate lablog_lib_v2 as v2;
extern crate lablog_store as store;

use self::chrono::DateTime;
use self::chrono::NaiveDateTime;
use self::chrono::Utc;
use self::store::note::Note;
use self::store::project_name::ProjectName;
use self::store::store::Store;
use self::v2::ProjectsNotes;
use errors::*;

pub fn write_v2_notes<T: Store>(store: &T, project_notes: ProjectsNotes) -> Result<()> {
    for (project, notes) in project_notes {
        let project_name: ProjectName = project.into();
        debug!("converting {}", project_name);

        for note in notes {
            let time_stamp = {
                let naive = NaiveDateTime::from_timestamp(
                    note.time_stamp.timestamp(),
                    note.time_stamp.timestamp_subsec_nanos(),
                );

                DateTime::from_utc(naive, Utc)
            };

            if let Err(err) = store.write_note(
                &project_name,
                &Note {
                    time_stamp: time_stamp,
                    value: note.value,
                },
            ) {
                warn!("can not write note to store: {}", err)
            };
        }
    }

    Ok(())
}
