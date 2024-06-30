// 目前db完全由手动维护，因为结构比较奇特且rust并无很好用的orm库

// db表结构：
// 若干自定义歌单表(歌单名称即是表名)，若干音乐原始数据表，一个自定义歌单元数据表，一个数据库信息表(记录版本等，用于迁移)

// 自定义歌单: music_list(只存储音乐原始数据的索引)
// 音乐原始数据: music_data(存储音乐的真实数据，每个平台都不太一样)
// 自定义歌单元数据: music_list_metadata(记录所有自定义歌单的名称，简介，艺术图)

// 自定义歌单只储存音乐数据的引用，而音乐数据存储某个音源的音乐数据
// 之所以如此设计是为了最大限度的保存音源音乐本身的数据，只有这样才能保证Music Trait的功能实现完整
mod db_action;
mod music_action;
mod musicdata_action;
mod musiclist_action;
mod sql_musiclist;
mod util;

use lazy_static::lazy_static;
use sea_query::TableCreateStatement;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use sqlx::{any::AnyRow, Any, Pool};
use tokio::sync::Mutex;

use crate::{util::StrIden, Music};

// 此Trait只包含不符合Object Safe的方法，因此也只能手动为每一个平台的实现和匹配
// 特定平台的音乐必须实现此Trait以从创建表和从表中反序列化
pub trait ObjectUnsafeStore {
    // 从表的一行数据中反序列化出Music
    fn from_row(row: AnyRow, id: i64) -> Result<Music, anyhow::Error>;
    // 生成创建一个原始音乐数据表所需的Statement
    fn create_table_query() -> TableCreateStatement;
}

// 一个音乐引用，表示的自定义歌单对原始音乐数据歌单的引用
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ref {
    // 引用所在表名,目标表格名
    pub t: String,
    // 键名
    pub k: String,
    // 键值
    pub v: String,
}

pub const VERSION: StrIden = StrIden("Version");
pub const METADATA: StrIden = StrIden("MetaData");
pub const REFMETADATA: StrIden = StrIden("RefMetaData");
pub const REFNAME: StrIden = StrIden("RefName");
pub const REFARTPIC: StrIden = StrIden("RefArtPic");
pub const REFDESC: StrIden = StrIden("RefDesc");

pub const ID: StrIden = StrIden("Id");
pub const DEFAULTSOURCE: StrIden = StrIden("DefaultSource");
pub const REFS: StrIden = StrIden("Refs");
pub const INDEX: StrIden = StrIden("Index");

lazy_static! {
    // 全局数据库连接池, 用于所有的数据库操作
    // 当前只考虑当个数据库
    pub static ref POOL: Arc<Mutex<Option<Pool<Any>>>> = Arc::new(Mutex::new(None));
}

// 为防止使用时错误使用其他Factory的相近名称
// 因此使用一个空的Factory来限定调用时名称
pub struct SqlFactory;
