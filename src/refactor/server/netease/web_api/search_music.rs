use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub result: Result,
    pub code: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub search_qc_reminder: Value,
    pub songs: Vec<Song>,
    pub song_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub name: String,
    pub id: i64,
    pub pst: i64,
    pub t: i64,
    pub ar: Vec<Artist>,
    pub alia: Vec<String>,
    pub pop: f64,
    pub st: i64,
    pub rt: Option<String>,
    pub fee: i64,
    pub v: i64,
    pub crbt: Value,
    pub cf: String,
    pub al: Al,
    pub dt: i64,
    pub h: H,
    pub m: M,
    pub l: L,
    pub sq: Sq,
    pub hr: Option<Hr>,
    pub a: Value,
    pub cd: String,
    pub no: i64,
    pub rt_url: Value,
    pub ftype: i64,
    pub rt_urls: Vec<Value>,
    pub dj_id: i64,
    pub copyright: i64,
    #[serde(rename = "s_id")]
    pub s_id: i64,
    pub mark: i64,
    pub origin_cover_type: i64,
    pub origin_song_simple_data: Value,
    pub tag_pic_list: Value,
    pub resource_state: bool,
    pub version: i64,
    pub song_jump_info: Value,
    pub entertainment_tags: Value,
    pub single: i64,
    pub no_copyright_rcmd: Value,
    pub rtype: i64,
    pub rurl: Value,
    pub mst: i64,
    pub cp: i64,
    pub mv: i64,
    pub publish_time: i64,
    pub privilege: Privilege,
    #[serde(default)]
    pub tns: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub tns: Vec<Value>,
    pub alias: Vec<String>,
    pub alia: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Al {
    pub id: i64,
    pub name: String,
    pub pic_url: String,
    pub tns: Vec<Value>,
    #[serde(rename = "pic_str")]
    pub pic_str: String,
    pub pic: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct H {
    pub br: i64,
    pub fid: i64,
    pub size: i64,
    pub vd: f64,
    pub sr: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct M {
    pub br: i64,
    pub fid: i64,
    pub size: i64,
    pub vd: f64,
    pub sr: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L {
    pub br: i64,
    pub fid: i64,
    pub size: i64,
    pub vd: f64,
    pub sr: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sq {
    pub br: i64,
    pub fid: i64,
    pub size: i64,
    pub vd: f64,
    pub sr: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hr {
    pub br: i64,
    pub fid: i64,
    pub size: i64,
    pub vd: f64,
    pub sr: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Privilege {
    pub id: i64,
    pub fee: i64,
    pub payed: i64,
    pub st: i64,
    pub pl: i64,
    pub dl: i64,
    pub sp: i64,
    pub cp: i64,
    pub subp: i64,
    pub cs: bool,
    pub maxbr: i64,
    pub fl: i64,
    pub toast: bool,
    pub flag: i64,
    pub pre_sell: bool,
    pub play_maxbr: i64,
    pub download_maxbr: i64,
    pub max_br_level: String,
    pub play_max_br_level: String,
    pub download_max_br_level: String,
    pub pl_level: String,
    pub dl_level: String,
    pub fl_level: String,
    pub rscl: i64,
    pub free_trial_privilege: FreeTrialPrivilege,
    pub right_source: i64,
    pub charge_info_list: Vec<ChargeInfoList>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreeTrialPrivilege {
    pub res_consumable: bool,
    pub user_consumable: bool,
    pub listen_type: Value,
    pub cannot_listen_reason: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargeInfoList {
    pub rate: i64,
    pub charge_url: Value,
    pub charge_message: Value,
    pub charge_type: i64,
}
