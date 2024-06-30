use sea_query::{ColumnDef, Expr, OnConflict, Query, TableCreateStatement};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{any::AnyRow, Row as _};

use crate::{
    factory::sql_factory::ObjectUnsafeStore,
    music_list::MusicList,
    util::{StrIden, StringIden},
    Music, MusicAggregator, MusicInfo, MusicInfoTrait, MusicTrait, ObjectSafeStore, Quality,
};

use super::{kuwo_album::get_music_album, kuwo_lyric::get_lrc, kuwo_quality::KuWoQuality, KUWO};

pub const ALBUM: &str = "Album";
pub const ALBUM_ID: &str = "Albumid";
pub const ARTIST: &str = "Artist";
pub const ARTIST_ID: &str = "Artistid";
pub const FORMAT: &str = "Format";
pub const SONG_NAME: &str = "Songname";
pub const MUSIC_RID: &str = "Musicrid";
pub const DURATION: &str = "Duration";
pub const QUALITY: &str = "Quality";
pub const DEFAULT_QUALITY: &str = "DefaultQuality";
pub const PIC: &str = "Pic";
pub const LYRIC: &str = "Lyric";

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    abslist: Vec<KuwoMusic>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KuwoMusic {
    #[serde(alias = "album")]
    #[serde(rename = "ALBUM")]
    pub(crate) album: String,

    #[serde(alias = "albumid")]
    #[serde(rename = "ALBUMID")]
    pub(crate) album_id: String,

    #[serde(rename = "ARTIST")]
    #[serde(alias = "artist")]
    pub(crate) artist: String,

    #[serde(alias = "artistid")]
    #[serde(rename = "ARTISTID")]
    pub(crate) artist_id: String,

    #[serde(rename = "FORMAT")]
    #[serde(default)]
    pub(crate) format: String,

    #[serde(rename = "SONGNAME")]
    #[serde(alias = "name")]
    pub(crate) song_name: String,

    #[serde(rename = "MUSICRID")]
    #[serde(alias = "id")]
    pub(crate) music_rid: String,

    #[serde(rename = "MINFO", default)]
    pub(crate) minfo: String,

    #[serde(rename = "N_MINFO", default)]
    pub(crate) n_minfo: String,

    #[serde(rename = "DURATION")]
    #[serde(alias = "duration")]
    pub(crate) duration: String,

    #[serde(default)]
    pub(crate) quality: Vec<KuWoQuality>,

    // 这个数据是由代码中默认给出的最高音质,与酷我无关
    #[serde(default)]
    pub(crate) default_quality: KuWoQuality,

    #[serde(default)]
    pub(crate) pic: String,

    #[serde(default)]
    pub(crate) lyric: String,

    // 注意这个id标志的是其在某自定义歌单中的主键的值,与酷我无关
    #[serde(default)]
    pub(crate) id: i64,
}

unsafe impl Sync for KuwoMusic {}
unsafe impl Send for KuwoMusic {}

impl ObjectUnsafeStore for KuwoMusic {
    // 实现从 sqlx::AnyRow 转换为 KuwoMusic 的方法
    fn from_row(row: AnyRow, id: i64) -> Result<Music, anyhow::Error> {
        Ok(Box::new(KuwoMusic {
            album: row.try_get(ALBUM).unwrap_or_default(),
            album_id: row.try_get(ALBUM_ID).unwrap_or_default(),
            artist: row.try_get(ARTIST).unwrap_or_default(),
            artist_id: row.try_get(ARTIST_ID).unwrap_or_default(),
            format: row.try_get(FORMAT).unwrap_or_default(),
            song_name: row.try_get(SONG_NAME).unwrap_or_default(),
            music_rid: row.try_get(MUSIC_RID).unwrap_or_default(),
            minfo: String::with_capacity(0),
            n_minfo: String::with_capacity(0),
            duration: row.try_get(DURATION).unwrap_or_default(),
            quality: serde_json::from_str::<Vec<KuWoQuality>>(&row.try_get::<String, _>(QUALITY)?)
                .unwrap_or_default(),
            pic: row.try_get(PIC).unwrap_or_default(),
            lyric: row.try_get(LYRIC).unwrap_or_default(),
            id,
            default_quality: serde_json::from_str::<KuWoQuality>(
                &row.try_get::<String, _>(DEFAULT_QUALITY)?,
            )
            .unwrap_or_default(),
        }))
    }
    fn create_table_query() -> TableCreateStatement {
        TableCreateStatement::new()
            .table(StrIden(KUWO))
            .col(ColumnDef::new(StrIden(&ALBUM)).string())
            .col(ColumnDef::new(StrIden(&ALBUM_ID)).string())
            .col(ColumnDef::new(StrIden(&ARTIST)).string())
            .col(ColumnDef::new(StrIden(&ARTIST_ID)).string())
            .col(ColumnDef::new(StrIden(&FORMAT)).string())
            .col(ColumnDef::new(StrIden(&SONG_NAME)).string())
            .col(
                ColumnDef::new(StrIden(&MUSIC_RID))
                    .string()
                    .primary_key()
                    .not_null(),
            )
            .col(ColumnDef::new(StrIden(&DURATION)).string())
            .col(ColumnDef::new(StrIden(&QUALITY)).string())
            .col(ColumnDef::new(StrIden(&PIC)).string())
            .col(ColumnDef::new(StrIden(&LYRIC)).string())
            .col(ColumnDef::new(StrIden(&DEFAULT_QUALITY)).string())
            .to_owned()
    }
}

impl MusicInfoTrait for KuwoMusic {
    fn get_primary_kv(&self) -> (String, String) {
        (MUSIC_RID.to_string(), self.music_rid.to_string())
    }

    fn get_music_info(&self) -> crate::MusicInfo {
        crate::MusicInfo {
            source: KUWO,
            name: self.song_name.clone(),
            artist: self.artist.split("&").map(|a| a.to_string()).collect(),
            duration: {
                match self.duration.parse() {
                    Ok(d) => Some(d),
                    _ => None,
                }
            },
            album: Some(self.album.clone()),
            qualities: self
                .quality
                .iter()
                .map(|quality| Quality {
                    level: Some(quality.level.clone()),
                    bitrate: {
                        if quality.bitrate != 0 {
                            Some(quality.bitrate)
                        } else {
                            None
                        }
                    },
                    format: Some(quality.format.clone()),
                    size: Some(quality.size.clone()),
                    short: format!("{}k{}", quality.bitrate, quality.format),
                })
                .collect::<Vec<Quality>>(),
            art_pic: {
                if self.pic.is_empty() {
                    None
                } else {
                    Some(self.pic.clone())
                }
            },
            lyric: {
                if self.lyric.is_empty() {
                    None
                } else {
                    Some(self.lyric.clone())
                }
            },
            id: self.id,
            default_quality: Some(Quality {
                level: Some(self.default_quality.level.clone()),
                bitrate: match self.default_quality.bitrate {
                    0 => None,
                    _ => Some(self.default_quality.bitrate),
                },
                format: Some(self.default_quality.format.clone()),
                size: Some(self.default_quality.format.clone()),
                short: format!(
                    "{}k{}",
                    self.default_quality.bitrate, self.default_quality.format
                ),
            }),
        }
    }

    fn source(&self) -> &'static str {
        KUWO
    }
    fn get_extra_info(&self, quality: &Quality) -> String {
        serde_json::to_string(&json!({"music_rid":self.music_rid,"quality":quality})).unwrap()
    }

    fn fetch_lyric(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + Send>>
    {
        let v = self.get_primary_kv().1;
        Box::pin(async move { Ok(get_lrc(&v).await?) })
    }

    fn clone_(&self) -> Music {
        Box::new(self.clone())
    }

    fn fetch_album(
        &self,
        page: u32,
        limit: u32,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<(MusicList, Vec<MusicAggregator>), anyhow::Error>,
                > + Send,
        >,
    > {
        let album_id = self.album_id.clone();
        let album_name = self.album.clone();
        Box::pin(async move { Ok(get_music_album(&album_id, &album_name, page, limit).await?) })
    }
}

impl MusicTrait for KuwoMusic {}

impl ObjectSafeStore for KuwoMusic {
    fn to_json(&self) -> Result<std::string::String, anyhow::Error> {
        Ok(serde_json::to_string(&self).unwrap())
    }
    fn to_sql_insert(&self) -> sea_query::InsertStatement {
        let table_iden = StrIden(self.source());
        let query: sea_query::InsertStatement = Query::insert()
            .into_table(table_iden)
            .columns(vec![
                StrIden(ALBUM),
                StrIden(ALBUM_ID),
                StrIden(ARTIST),
                StrIden(ARTIST_ID),
                StrIden(FORMAT),
                StrIden(SONG_NAME),
                StrIden(MUSIC_RID),
                StrIden(DURATION),
                StrIden(QUALITY),
                StrIden(PIC),
                StrIden(LYRIC),
                StrIden(DEFAULT_QUALITY),
            ])
            .values_panic(vec![
                self.album.clone().into(),
                self.album_id.clone().into(),
                self.artist.clone().into(),
                self.artist_id.clone().into(),
                self.format.clone().into(),
                self.song_name.clone().into(),
                self.music_rid.clone().into(),
                self.duration.clone().into(),
                serde_json::to_string(&self.quality)
                    .expect("Failed to serialize quality")
                    .into(),
                self.pic.clone().into(),
                self.lyric.clone().into(),
                serde_json::to_string(&self.default_quality)
                    .expect("Failed to serialize quality")
                    .into(),
            ])
            .on_conflict(
                OnConflict::column(StrIden(MUSIC_RID))
                    .do_nothing()
                    .to_owned(),
            )
            .to_owned();
        query
    }

    fn to_sql_update(&self, info: &MusicInfo) -> Result<sea_query::UpdateStatement, anyhow::Error> {
        let origin = self.get_music_info();
        let (k, v) = self.get_primary_kv();
        let mut binding = Query::update().clone();
        let query = binding
            .table(StrIden(self.source()))
            .and_where(Expr::col(StringIden(k)).eq(v));
        let mut need_update = false;
        if origin.name != info.name {
            query.value(StrIden(&SONG_NAME), info.name.clone());
            need_update = true;
        }
        if origin.artist != info.artist {
            query.value(StrIden(&ARTIST), info.artist.clone().join("&"));
            need_update = true;
        }
        if origin.duration != info.duration {
            query.value(StrIden(&DURATION), info.duration);
            need_update = true;
        }
        if origin.album != info.album {
            query.value(StrIden(&ALBUM), info.album.clone());
            need_update = true;
        }
        if origin.art_pic != info.art_pic {
            query.value(StrIden(&PIC), info.art_pic.clone());
            need_update = true;
        }
        if origin.lyric != info.lyric {
            query.value(StrIden(&LYRIC), serde_json::to_string(&info.lyric).unwrap());
            need_update = true;
        }
        if origin.default_quality != info.default_quality {
            query.value(
                StrIden(&DEFAULT_QUALITY),
                serde_json::to_string(&info.default_quality).unwrap(),
            );
            need_update = true;
        }
        if need_update {
            Ok(query.clone())
        } else {
            Err(anyhow::anyhow!("No need to update"))
        }
    }
}
