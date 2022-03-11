use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Dictionary {
    pub words: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Daily {
    pub word: String,
    #[serde(with = "ts_milliseconds")]
    pub date: DateTime<Utc>,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Previous {
    pub previous: Vec<Daily>,
}
