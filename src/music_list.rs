use std::fmt::Display;

use sqlx::{any::AnyRow, Row};

use crate::sql_store_actory::{REFARTPIC, REFDESC, REFNAME};
#[derive(Debug)]
pub struct MusicList {
    pub name: String,
    pub art_pic: String,
    pub desc: String,
}
impl MusicList {
    pub fn from_row(row: AnyRow) -> Result<MusicList, anyhow::Error> {
        Ok(MusicList {
            name: row.try_get(REFNAME.0).unwrap_or("Unknown".to_string()),
            art_pic: row.try_get(REFARTPIC.0).unwrap_or_default(),
            desc: row.try_get(REFDESC.0).unwrap_or_default(),
        })
    }
}

impl Display for MusicList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}--{}--{}", self.name, self.desc, self.art_pic)
    }
}
