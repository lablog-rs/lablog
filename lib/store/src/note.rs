use chrono::prelude::*;
use std::collections::BTreeSet;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub struct Note {
    pub time_stamp: DateTime<Utc>,
    pub value: String,
}

impl From<String> for Note {
    fn from(input: String) -> Self {
        Note {
            value: input,
            time_stamp: Utc::now(),
        }
    }
}

pub type Notes = BTreeSet<Note>;
