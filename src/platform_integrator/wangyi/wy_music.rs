#![allow(non_snake_case, unused)]
use std::{clone, default, pin::Pin};

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

use super::{get_musics_from_album, wy_lyric::get_lyric, WANGYI};
pub const ID_: &str = "ID_";
pub const NAME: &str = "Name";
pub const ARTPIC: &str = "ArtPic";
pub const LYRIC: &str = "Lyric";
pub const ID: &str = "ID";
pub const ARTISTS: &str = "Artists";
pub const ALBUM: &str = "Album";
pub const DURATION: &str = "Duration";
pub const WY_TABLE: &str = "WangYi";
pub const H: &str = "H";
pub const M: &str = "M";
pub const L: &str = "L";
pub const SQ: &str = "SQ";
pub const HR: &str = "HR";
pub const DEFAULT_QUALITY: &str = "DefaultQuality";

impl ObjectSafeStore for WyMusic {
    fn to_json(&self) -> Result<String, anyhow::Error> {
        Ok(serde_json::to_string(&self)?)
    }

    fn to_sql_insert(&self) -> sea_query::InsertStatement {
        let table_iden = StringIden(self.source()); // 使用全局变量定义的表名
        let qualities = self.get_qualities();
        let default_quality = if let Some(quality) = qualities.first() {
            Some(quality.clone())
        } else {
            None
        };
        let query = sea_query::Query::insert()
            .into_table(table_iden)
            .columns(vec![
                StrIden(NAME),
                StrIden(ID),
                StrIden(ARTPIC),
                StrIden(ARTISTS),
                StrIden(DURATION),
                StrIden(ALBUM),
                StrIden(LYRIC),
                StrIden(H),
                StrIden(M),
                StrIden(L),
                StrIden(SQ),
                StrIden(HR),
                StrIden(DEFAULT_QUALITY),
            ])
            .values_panic(vec![
                self.name.clone().into(),
                self.id.into(),
                self.artpic.clone().into(),
                serde_json::to_string(&self.ar).unwrap_or_default().into(),
                self.dt.into(),
                serde_json::to_string(&self.al).unwrap_or_default().into(),
                self.lyric.clone().into(),
                serde_json::to_string(&self.h).unwrap_or_default().into(),
                serde_json::to_string(&self.m).unwrap_or_default().into(),
                serde_json::to_string(&self.l).unwrap_or_default().into(),
                serde_json::to_string(&self.sq).unwrap_or_default().into(),
                serde_json::to_string(&self.hr).unwrap_or_default().into(),
                serde_json::to_string(&default_quality)
                    .unwrap_or_default()
                    .into(),
            ])
            .on_conflict(OnConflict::column(StrIden(ID)).do_nothing().to_owned())
            .to_owned();
        query
    }

    fn to_sql_update(
        &self,
        info: &crate::MusicInfo,
    ) -> Result<sea_query::UpdateStatement, anyhow::Error> {
        let origin = self.get_music_info();
        let (k, v) = self.get_primary_kv();
        let mut binding = Query::update().clone();
        let query = binding
            .table(StringIden(self.source()))
            .and_where(Expr::col(StringIden(k.to_string())).eq(v));
        let mut need_update = false;

        if origin.name != info.name {
            query.value(StrIden(NAME), info.name.clone());
            need_update = true;
        }
        if origin.artist != info.artist {
            query.value(
                StrIden(ARTISTS),
                serde_json::to_string(&info.artist).unwrap(),
            );
            need_update = true;
        }
        if origin.duration != info.duration {
            query.value(StrIden(&DURATION), info.duration);
            need_update = true;
        }
        if origin.album != info.album {
            query.value(StrIden(ALBUM), info.album.clone());
            need_update = true;
        }
        if origin.art_pic != info.art_pic {
            if let Some(art_pic) = &info.art_pic {
                query.value(StrIden(&ARTPIC), art_pic.clone());
            }
            need_update = true;
        }
        if origin.lyric != info.lyric {
            if let Some(lyric) = &info.lyric {
                query.value(StrIden(&LYRIC), lyric.clone());
            }
            need_update = true;
        }

        if info.default_quality.is_some() && origin.default_quality != info.default_quality {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AlbumPayload {
    pub id: Option<u64>,
}

impl MusicInfoTrait for WyMusic {
    fn source(&self) -> String {
        WANGYI.to_string()
    }

    fn get_music_info(&self) -> crate::MusicInfo {
        let qualities = self.get_qualities();
        MusicInfo {
            id: self.id_,
            source: WANGYI.to_string(),
            name: self.name.to_string(),
            artist: self.ar.iter().map(|a| a.name.clone()).collect(),
            duration: Some(self.dt.try_into().unwrap_or(0) / 1000),
            album: {
                if let Some(al) = &self.al {
                    Some(al.name.to_string())
                } else {
                    None
                }
            },
            default_quality: self.default_quality.clone(),
            qualities: qualities,
            art_pic: {
                if let Some(art_pic) = &self.artpic {
                    Some(art_pic.clone())
                } else {
                    if let Some(album_pic) = &self.al {
                        album_pic.picUrl.clone()
                    } else {
                        None
                    }
                }
            },
            lyric: self.lyric.clone(),
        }
    }

    fn get_extra_info(&self, quality: &crate::Quality) -> String {
        json!({
            "id": self.id,
            "quality": quality.short,
        })
        .to_string()
    }

    fn get_primary_kv(&self) -> (String, String) {
        (ID.to_string(), self.id.to_string())
    }

    fn fetch_lyric(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + Send>> {
        let v = self.get_primary_kv().1;
        Box::pin(async move { Ok(get_lyric(&v).await?) })
    }

    fn fetch_album(
        &self,
        _page: u32,
        _limit: u32,
    ) -> Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<(MusicList, Vec<MusicAggregator>), anyhow::Error>,
                > + Send,
        >,
    > {
        let album_id_option = self.al.as_ref().map(|al| al.id);
        Box::pin(async move {
            let id = album_id_option.ok_or(anyhow::anyhow!("No album id"))?;
            Ok(get_musics_from_album(id).await?)
        })
    }

    fn clone_(&self) -> Music {
        Box::new(self.clone())
    }
}

impl MusicTrait for WyMusic {}

impl ObjectUnsafeStore for WyMusic {
    fn from_row(row: AnyRow, id: i64) -> Result<crate::Music, anyhow::Error> {
        Ok(Box::new(WyMusic {
            id_: id,
            name: row.try_get(NAME).unwrap_or_default(),
            artpic: row.try_get(ARTPIC).unwrap_or_default(),
            ar: serde_json::from_str(&row.try_get::<String, _>(ARTISTS).unwrap_or_default())
                .unwrap_or_default(),
            al: serde_json::from_str(&row.try_get::<String, _>(ALBUM).unwrap_or_default())
                .unwrap_or_default(),
            dt: row.try_get(DURATION).unwrap_or_default(),
            id: row.try_get(ID).unwrap_or_default(),
            lyric: row.try_get(LYRIC).unwrap_or_default(),
            h: row
                .try_get::<String, _>(H)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            m: row
                .try_get::<String, _>(M)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            l: row
                .try_get::<String, _>(L)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            sq: row
                .try_get::<String, _>(SQ)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            hr: row
                .try_get::<String, _>(HR)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_quality: row
                .try_get::<String, _>(DEFAULT_QUALITY)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok()),
        }))
    }

    fn create_table_query() -> sea_query::TableCreateStatement {
        TableCreateStatement::new()
            .table(StrIden(WY_TABLE))
            .col(ColumnDef::new(StrIden(NAME)).string().not_null())
            .col(
                ColumnDef::new(StrIden(ID))
                    .big_integer()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(StrIden(ARTPIC)).string().null())
            .col(ColumnDef::new(StrIden(ARTISTS)).string().null())
            .col(ColumnDef::new(StrIden(ALBUM)).string().null())
            .col(ColumnDef::new(StrIden(DURATION)).integer().null())
            .col(ColumnDef::new(StrIden(LYRIC)).string().null())
            .col(ColumnDef::new(StrIden(H)).string().null())
            .col(ColumnDef::new(StrIden(M)).string().null())
            .col(ColumnDef::new(StrIden(L)).string().null())
            .col(ColumnDef::new(StrIden(SQ)).string().null())
            .col(ColumnDef::new(StrIden(HR)).string().null())
            .col(ColumnDef::new(StrIden(DEFAULT_QUALITY)).string().null())
            .to_owned()
    }
}

impl Default for WyMusic {
    fn default() -> Self {
        WyMusic {
            id_: 0,
            name: "".to_string(),
            id: 0,
            ar: Vec::new(),
            al: None,
            dt: 0,
            h: None,
            m: None,
            l: None,
            sq: None,
            hr: None,
            artpic: None,
            lyric: None,
            default_quality: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WyMusic {
    // 自定义歌单内部的主键id，用于数据库操作
    #[serde(default)]
    id_: i64,
    // 歌曲名称
    name: String,
    // 网易云音乐的歌曲id
    id: i64,
    // 演唱者
    ar: Vec<Artist>,
    // 专辑信息
    al: Option<Album>,
    // 歌曲时长
    dt: i32,
    // 一系列的音频质量
    h: Option<AudioQuality>,
    m: Option<AudioQuality>,
    l: Option<AudioQuality>,
    sq: Option<AudioQuality>,
    hr: Option<AudioQuality>,
    #[serde(default)]
    artpic: Option<String>,
    #[serde(default)]
    lyric: Option<String>,
    pub default_quality: Option<Quality>,
    // 不清楚这些字段的含义
    // rt: Option<String>,
    // crbt: Option<String>,
    // resourceState: bool,
    // mv: u64,
    // cp: u32,
    // no: u32,
    // mark: u64,
    // version: u32,
    // pst: u32,
    // t: u32,
    // alia: Vec<String>,
    // pop: f32,
    // st: u32,
    // fee: u32,
    // v: u32,
    // cd: String,
    // rtUrl: Option<String>,
    // ftype: u32,
    // djId: u64,
    // copyright: u32,
    // s_id: u32,
    // rtUrls: Vec<String>,
    // originCoverType: u32,
    // originSongSimpleData: Option<String>,
    // tagPicList: Option<String>,
    // songJumpInfo: Option<String>,
    // entertainmentTags: Option<String>,
    // single: u32,
    // noCopyrightRcmd: Option<String>,
    // rtype: u32,
    // rurl: Option<String>,
    // mst: u32,
    // publishTime: i64,
    // privilege: Privilege,
    // cf: String,
    // a: Option<String>,
}

unsafe impl Sync for WyMusic {}
unsafe impl Send for WyMusic {}
impl WyMusic {
    pub fn get_qualities(&self) -> Vec<Quality> {
        let mut qualities = Vec::new();
        if let Some(hr) = &self.hr {
            qualities.push(Quality {
                short: "hires".to_string(),
                level: Some(hr.sr.to_string()),
                bitrate: Some(hr.br),
                format: Some("unknown".to_string()),
                size: Some(hr.size.to_string()),
            });
        }
        if let Some(sq) = &self.sq {
            qualities.push(Quality {
                short: "lossless".to_string(),
                level: Some(sq.sr.to_string()),
                bitrate: Some(sq.br),
                format: Some("unknown".to_string()),
                size: Some(sq.size.to_string()),
            });
        }
        if let Some(h) = &self.h {
            qualities.push(Quality {
                short: "exhigh".to_string(),
                level: Some(h.sr.to_string()),
                bitrate: Some(h.br),
                format: Some("unknown".to_string()),
                size: Some(h.size.to_string()),
            });
        }
        if let Some(m) = &self.m {
            qualities.push(Quality {
                short: "higher".to_string(),
                level: Some(m.sr.to_string()),
                bitrate: Some(m.br),
                format: Some("unknown".to_string()),
                size: Some(m.size.to_string()),
            });
        }
        if let Some(l) = &self.l {
            qualities.push(Quality {
                short: "standard".to_string(),
                level: Some(l.sr.to_string()),
                bitrate: Some(l.br),
                format: Some("unknown".to_string()),
                size: Some(l.size.to_string()),
            });
        }
        qualities
    }
    // 获取最高的音质
    pub fn get_highest_quality(&self) -> Option<Quality> {
        if let Some(hr) = &self.hr {
            return Some(Quality {
                short: "hires".to_string(),
                level: Some(hr.sr.to_string()),
                bitrate: Some(hr.br),
                format: Some("unknown".to_string()),
                size: Some(hr.size.to_string()),
            });
        }
        if let Some(sq) = &self.sq {
            return Some(Quality {
                short: "super".to_string(),
                level: Some(sq.sr.to_string()),
                bitrate: Some(sq.br),
                format: Some("unknown".to_string()),
                size: Some(sq.size.to_string()),
            });
        }
        if let Some(h) = &self.h {
            return Some(Quality {
                short: "high".to_string(),
                level: Some(h.sr.to_string()),
                bitrate: Some(h.br),
                format: Some("unknown".to_string()),
                size: Some(h.size.to_string()),
            });
        }
        if let Some(m) = &self.m {
            return Some(Quality {
                short: "medium".to_string(),
                level: Some(m.sr.to_string()),
                bitrate: Some(m.br),
                format: Some("unknown".to_string()),
                size: Some(m.size.to_string()),
            });
        }
        if let Some(l) = &self.l {
            return Some(Quality {
                short: "low".to_string(),
                level: Some(l.sr.to_string()),
                bitrate: Some(l.br),
                format: Some("unknown".to_string()),
                size: Some(l.size.to_string()),
            });
        }
        None
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Artist {
    id: u64,
    name: String,
    #[serde(default)]
    tns: Vec<String>,
    #[serde(default)]
    alias: Vec<String>,
    #[serde(default)]
    alia: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Album {
    id: u64,
    name: String,
    picUrl: Option<String>,
    #[serde(default)]
    tns: Vec<String>,
    pic_str: Option<String>,
    pic: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AudioQuality {
    br: u32,
    fid: u32,
    size: u32,
    vd: f32,
    #[serde(default)]
    sr: u32,
}

// #[derive(Debug, Deserialize, Serialize)]
// struct Privilege {
//     id: u64,
//     fee: u32,
//     payed: u32,
//     st: u32,
//     pl: u32,
//     dl: u32,
//     sp: u32,
//     cp: u32,
//     subp: u32,
//     cs: bool,
//     maxbr: u32,
//     fl: u32,
//     toast: bool,
//     flag: u32,
//     preSell: bool,
//     playMaxbr: u32,
//     downloadMaxbr: u32,
//     maxBrLevel: String,
//     playMaxBrLevel: String,
//     downloadMaxBrLevel: String,
//     plLevel: String,
//     dlLevel: String,
//     flLevel: String,
//     rscl: Option<String>,
//     freeTrialPrivilege: FreeTrialPrivilege,
//     rightSource: u32,
//     chargeInfoList: Vec<ChargeInfo>,
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct FreeTrialPrivilege {
//     resConsumable: bool,
//     userConsumable: bool,
//     listenType: u32,
//     cannotListenReason: u32,
// }

// #[derive(Debug, Deserialize, Serialize)]
// struct ChargeInfo {
//     rate: u32,
//     chargeUrl: Option<String>,
//     chargeMessage: Option<String>,
//     chargeType: u32,
// }

#[test]
fn main() {
    let json_data = r#"
        {
            "name": "我可以抱你吗(Live)",
            "id": 326843,
            "pst": 0,
            "t": 0,
            "ar": [
                {
                    "id": 10559,
                    "name": "张惠妹",
                    "tns": [],
                    "alias": [
                        "aMEI",
                        "阿妹",
                        "阿密特"
                    ],
                    "alia": [
                        "aMEI",
                        "阿妹",
                        "阿密特"
                    ]
                }
            ],
            "alia": [],
            "pop": 100,
            "st": 0,
            "rt": "600902000009487414",
            "fee": 8,
            "v": 590,
            "crbt": null,
            "cf": "",
            "al": {
                "id": 32329,
                "name": "阿密特 首次世界巡回演唱会",
                "picUrl": "http://p2.music.126.net/5fCcQpRl2UNq8XO3U9ngWg==/109951163265788528.jpg",
                "tns": [],
                "pic_str": "109951163265788528",
                "pic": 109951163265788530
            },
            "dt": 291064,
            "h": {
                "br": 320000,
                "fid": 0,
                "size": 11645431,
                "vd": 17156,
                "sr": 44100
            },
            "m": {
                "br": 192000,
                "fid": 0,
                "size": 6987276,
                "vd": 19787,
                "sr": 44100
            },
            "l": {
                "br": 128000,
                "fid": 0,
                "size": 4658199,
                "vd": 21516,
                "sr": 44100
            },
            "sq": {
                "br": 821686,
                "fid": 0,
                "size": 29895520,
                "vd": 17157,
                "sr": 48000
            },
            "hr": null,
            "a": null,
            "cd": "1",
            "no": 14,
            "rtUrl": null,
            "ftype": 0,
            "rtUrls": [],
            "djId": 0,
            "copyright": 1,
            "s_id": 0,
            "mark": 17179877376,
            "originCoverType": 1,
            "originSongSimpleData": null,
            "tagPicList": null,
            "resourceState": true,
            "version": 590,
            "songJumpInfo": null,
            "entertainmentTags": null,
            "single": 0,
            "noCopyrightRcmd": null,
            "rtype": 0,
            "rurl": null,
            "mst": 9,
            "cp": 13009,
            "mv": 5570452,
            "publishTime": 1277395200000,
            "privilege": {
                "id": 326843,
                "fee": 8,
                "payed": 0,
                "st": 0,
                "pl": 128000,
                "dl": 0,
                "sp": 7,
                "cp": 1,
                "subp": 1,
                "cs": false,
                "maxbr": 999000,
                "fl": 320000,
                "toast": false,
                "flag": 260,
                "preSell": false,
                "playMaxbr": 999000,
                "downloadMaxbr": 999000,
                "maxBrLevel": "lossless",
                "playMaxBrLevel": "lossless",
                "downloadMaxBrLevel": "lossless",
                "plLevel": "standard",
                "dlLevel": "none",
                "flLevel": "exhigh",
                "rscl": null,
                "freeTrialPrivilege": {
                    "resConsumable": false,
                    "userConsumable": false,
                    "listenType": 0,
                    "cannotListenReason": 1
                },
                "rightSource": 0,
                "chargeInfoList": [
                    {
                        "rate": 128000,
                        "chargeUrl": null,
                        "chargeMessage": null,
                        "chargeType": 0
                    },
                    {
                        "rate": 192000,
                        "chargeUrl": null,
                        "chargeMessage": null,
                        "chargeType": 0
                    },
                    {
                        "rate": 320000,
                        "chargeUrl": null,
                        "chargeMessage": null,
                        "chargeType": 0
                    },
                    {
                        "rate": 999000,
                        "chargeUrl": null,
                        "chargeMessage": null,
                        "chargeType": 1
                    }
                ]
            }
        }
    "#;

    let song: WyMusic = serde_json::from_str(json_data).unwrap();
    println!("{:?}", song);
}
