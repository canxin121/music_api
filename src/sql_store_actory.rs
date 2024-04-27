// 音乐数据sql储存设计

// 自定义歌单: music_list
// 音乐数据: music_data
// 自定义歌单元数据: music_list_metadata
// 自定义歌单元数据储存自定义歌单本身的数据，自定义歌单只储存音乐数据的引用，而音乐数据存储某个音源的音乐数据
// 之所以如此设计是为了最大限度的保存音源音乐本身的数据，只有这样才能保证Music Trait的功能实现完整

use sea_query::{
    Asterisk, ColumnDef, Cond, Expr, InsertStatement, Query, Table, TableCreateStatement,
};
use serde::{Deserialize, Serialize};

use sqlx::{any::AnyRow, Acquire as _, Any, AnyPool, Pool, Row as _};

use crate::{
    kuwo::{
        self,
        kuwo_music::{KuwoMusic, MUSIC_RID},
        KUWO,
    },
    music_list::MusicList,
    util::{build_query, build_sqlx_query, StrIden, StringIden},
    Music, MusicInfo,
};

pub trait SqlFactoryStore {
    fn from_row(row: AnyRow, index: i64) -> Result<Music, anyhow::Error>;
    fn create_table_query() -> TableCreateStatement;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ref {
    // 引用所在表名
    pub t: String,
    // 键名
    pub k: String,
    // 键值
    pub v: String,
}
pub const REFMETADATA: StrIden = StrIden("RefMetaData");
pub const REFNAME: StrIden = StrIden("RefName");
pub const REFARTPIC: StrIden = StrIden("RefArtPic");
pub const REFDESC: StrIden = StrIden("RefDesc");

pub const ID: StrIden = StrIden("Id");
pub const DEFAULTSOURCE: StrIden = StrIden("DefaultSource");
pub const REFS: StrIden = StrIden("Refs");
pub const INDEX: StrIden = StrIden("Index");
pub struct SqlMusicFactory {
    pool: Pool<Any>,
}

impl SqlMusicFactory {
    pub fn new(pool: AnyPool) -> SqlMusicFactory {
        SqlMusicFactory { pool }
    }
    // 插入音乐数据
    async fn insert_music_data(&self, musics: &Vec<&Music>) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;

        for music in musics {
            let query = music.to_sql_insert();
            let (insert_sql, insert_values) = build_sqlx_query(query).await?;
            sqlx::query_with(&insert_sql, insert_values)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // 创建自定义歌单元数据表
    async fn create_music_list_metadata_table(&self) -> Result<(), anyhow::Error> {
        let query = TableCreateStatement::new()
            .table(REFMETADATA)
            .col(ColumnDef::new(REFNAME).string().not_null())
            .col(ColumnDef::new(REFARTPIC).string().not_null())
            .col(ColumnDef::new(REFDESC).integer())
            .col(ColumnDef::new(ID).integer().primary_key().auto_increment())
            .clone();
        let mut conn = self.pool.acquire().await?;
        let s: String = build_query(query).await?;
        sqlx::query(&s).execute(&mut *conn).await?;
        Ok(())
    }
    // 初始化操作，创建所有原始数据表
    async fn create_music_data_table(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;
        let query1 = KuwoMusic::create_table_query();
        let s1: String = build_query(query1).await?;
        sqlx::query(&s1).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    // 将自定义歌单信息插入插入自定义歌单元数据表
    // 删除自定义歌单的元数据
    async fn del_music_list_metadata(
        &self,
        music_list: &Vec<MusicList>,
    ) -> Result<(), anyhow::Error> {
        let mut cond = Cond::any();
        for r in music_list {
            cond = cond.add(Expr::col(REFNAME).eq(r.name.clone()));
        }
        let query = Query::delete()
            .from_table(REFMETADATA)
            .cond_where(cond)
            .to_owned();
        let mut conn = self.pool.acquire().await?;
        let mut tx: sqlx::Transaction<'_, sqlx::Any> = conn.begin().await?;

        let (delete_sql, delete_values) = build_sqlx_query(query).await?;
        sqlx::query_with(&delete_sql, delete_values)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    //// SqlMusic的外部可用方法

    /// 对自定义歌单的操作

    async fn insert_music_list_matadata(
        &self,
        music_lists: &Vec<MusicList>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;
        for music_list in music_lists {
            let query = InsertStatement::new()
                .into_table(REFMETADATA)
                .columns(vec![REFNAME, REFDESC, REFARTPIC])
                .values_panic(vec![
                    music_list.name.clone().into(),
                    music_list.desc.clone().into(),
                    music_list.art_pic.clone().into(),
                ])
                .to_owned();

            let (insert_sql, insert_values) = build_sqlx_query(query).await?;
            sqlx::query_with(&insert_sql, insert_values)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // 创建一个自定义歌单表
    pub async fn create_music_list_table(
        &self,
        music_lists: &Vec<MusicList>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;
        for music_list in music_lists {
            let query = TableCreateStatement::new()
                .table(StringIden(music_list.name.to_string()))
                .col(
                    ColumnDef::new(ID)
                        .integer()
                        .auto_increment()
                        .primary_key()
                        .not_null(),
                )
                .col(ColumnDef::new(DEFAULTSOURCE).string().not_null())
                .col(ColumnDef::new(REFS).string().not_null())
                .col(ColumnDef::new(INDEX).integer())
                .clone();
            let s: String = build_query(query).await?;
            sqlx::query(&s).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        // 最后插入元数据
        self.insert_music_list_matadata(music_lists).await?;
        Ok(())
    }

    // 修改自定义歌单元数据
    pub async fn change_music_list_metadata(
        &self,
        old: &Vec<MusicList>,
        new: &Vec<MusicList>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;
        for (old_one, new_one) in old.iter().zip(new) {
            let mut query = Query::update().table(REFMETADATA).to_owned();
            let mut need_update = false;
            let mut need_rename = false;
            if new_one.name != old_one.name && !new_one.name.is_empty() {
                query.value::<StrIden, String>(REFNAME, new_one.name.to_string().into());
                need_rename = true;
                need_update = true;
            }
            if new_one.art_pic != old_one.art_pic && !new_one.art_pic.is_empty() {
                query.value::<StrIden, String>(REFARTPIC, new_one.art_pic.to_string().into());
                need_update = true;
            }
            if new_one.desc != old_one.desc {
                query.value::<StrIden, String>(REFDESC, new_one.desc.to_string().into());
                need_update = true;
            }

            if !need_update {
                return Ok(());
            }

            if need_rename {
                let rename_query = Table::rename()
                    .table(
                        StringIden(old_one.name.clone()),
                        StringIden(new_one.name.clone()),
                    )
                    .clone();
                let s = build_query(rename_query).await?;
                match sqlx::query(&s).execute(&mut *tx).await {
                    Ok(_) => {
                        query.and_where(Expr::col(REFNAME).eq(old_one.name.clone()));
                        let (s, q) = build_sqlx_query(query).await?;
                        sqlx::query_with(&s, q).execute(&mut *tx).await?;
                    }
                    Err(e) => {
                        println!("Failed to rename table:{e}")
                    }
                };
            } else {
                query.and_where(Expr::col(REFNAME).eq(old_one.name.clone()));
                let (s, q) = build_sqlx_query(query).await?;
                sqlx::query_with(&s, q).execute(&mut *tx).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    // 获取所有自定义歌单
    pub async fn read_music_lists(&self) -> Result<Vec<MusicList>, anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        // 先获取所有的ref数据
        let music_list_query = Query::select()
            .from(REFMETADATA)
            .columns(vec![REFNAME, REFARTPIC, REFDESC])
            .clone();
        let (music_list_sql, music_list_values) = build_sqlx_query(music_list_query).await?;

        let music_list_results = sqlx::query_with(&music_list_sql, music_list_values)
            .fetch_all(&mut *conn)
            .await?;
        let mut results = Vec::new();
        for result in music_list_results {
            if let Ok(music_list_) = MusicList::from_row(result) {
                results.push(music_list_);
            }
        }
        Ok(results)
    }

    // 删除一个自定义歌单
    pub async fn del_music_list_table(
        &self,
        music_lists: &Vec<MusicList>,
    ) -> Result<(), anyhow::Error> {
        // 先删去元数据
        self.del_music_list_metadata(music_lists).await?;

        let mut query = Table::drop().to_owned();
        music_lists.iter().for_each(|r| {
            query.table(StringIden(r.name.to_string()));
        });
        let mut conn = self.pool.acquire().await?;

        let s = build_query(query).await?;
        sqlx::query(&s).execute(&mut *conn).await?;
        Ok(())
    }

    /// 对自定义歌单内部的音乐的操作

    // 向自定义歌单插入音乐
    pub async fn insert_music(
        &self,
        music_list: &MusicList,
        musics: &Vec<&Music>,
    ) -> Result<(), anyhow::Error> {
        // Check if the lengths of index and musics match
        // 先插入到原始数据表中
        self.insert_music_data(musics).await?;

        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;

        let result = sqlx::query(&format!("SELECT MAX(Id) FROM '{}'", music_list.name))
            .fetch_one(&mut *tx)
            .await?;
        let max_id: i64 = result.try_get(0).unwrap_or(0);

        let indexs: Vec<i64> = (max_id + 1..max_id + 1 + musics.len() as i64).collect();

        for (music, index) in musics.iter().zip(indexs) {
            let (k, v) = music.get_primary_kv();
            let source = music.source();
            let music_list_ = vec![Ref {
                t: source.to_owned(),
                k,
                v,
            }];
            let query = InsertStatement::new()
                .into_table(StringIden(music_list.name.to_string()))
                .columns(vec![REFS, DEFAULTSOURCE, INDEX])
                .values_panic(vec![
                    serde_json::to_string(&music_list_).unwrap().into(),
                    music.source().into(),
                    index.into(),
                ])
                .to_owned();
            let (insert_sql, insert_values) = build_sqlx_query(query).await?;
            sqlx::query_with(&insert_sql, insert_values)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // 删除自定义歌单中的音乐
    pub async fn del_music(
        &self,
        music_list: &MusicList,
        ids: Vec<i64>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let mut tx: sqlx::Transaction<'_, sqlx::Any> = conn.begin().await?;

        for id in ids {
            let delete_query = Query::delete()
                .from_table(StringIden(music_list.name.to_string()))
                .cond_where(Cond::any().add(Expr::col(ID).eq(id)))
                .to_owned();

            let (delete_sql, delete_values) = build_sqlx_query(delete_query).await?;
            sqlx::query_with(&delete_sql, delete_values)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // 更改自定义歌单的歌曲顺序
    pub async fn reorder_music(
        &self,
        music_list: &MusicList,
        new_index: Vec<i64>,
        old_musics_in_order: &Vec<&Music>,
    ) -> Result<(), anyhow::Error> {
        // 排序过程概述
        // 传入的old_music_in_order本身是按照index从小到大排序好的，但是这里无法得知到底是几，只知道大小关系
        // 注意old_music_in_order中的music的id是其在music_list中的主键，而不是index顺序
        // 传入的new_index是对应位置的music的新的index值

        // 所以排序只需要，拿到这个位置的music的新的index值，然后其更新到这个music的index上就好了
        // 而查找这个music的时候，只需要使用主键id就可以了

        if new_index.len() != old_musics_in_order.len() {
            return Err(anyhow::Error::msg("Index and musics length mismatch"));
        }

        let mut conn = self.pool.acquire().await?;

        let mut tx = conn.begin().await?;

        for (new_index, old_music) in new_index.into_iter().zip(old_musics_in_order.into_iter()) {
            let id = old_music.get_music_id();
            let update_query = Query::update()
                .table(StringIden(music_list.name.to_string()))
                .value(INDEX, new_index)
                .and_where(Expr::col(ID).eq(id))
                .to_owned();

            let (update_sql, update_values) = build_sqlx_query(update_query).await?;

            sqlx::query_with(&update_sql, update_values)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    // 读取自定义歌单的所有音乐
    pub async fn read_music(&self, music_list: &MusicList) -> Result<Vec<Music>, anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        // 先获取所有的ref数据
        let music_list_query = Query::select()
            .from(StringIden(music_list.name.to_string()))
            .columns(vec![ID, INDEX, DEFAULTSOURCE, REFS])
            .clone();
        let (music_list_sql, music_list_values) = build_sqlx_query(music_list_query).await?;

        let music_list_results = sqlx::query_with(&music_list_sql, music_list_values)
            .fetch_all(&mut *conn)
            .await?;

        let mut music_indexs: Vec<(Music, i64)> = Vec::new();

        for music_list_result in music_list_results.iter() {
            // 仅用于排序
            let index_: i64 = music_list_result.try_get(INDEX.0)?;

            let default_source: String = music_list_result.try_get(DEFAULTSOURCE.0)?;
            let refs: String = music_list_result.try_get(REFS.0)?;
            let id: i64 = music_list_result.try_get(ID.0)?;
            let music_list_ = serde_json::from_str::<Vec<Ref>>(&refs)?;
            let music_list_one = match music_list_
                .into_iter()
                .find(|music_list_| music_list_.t == default_source)
            {
                Some(r) => r,
                None => continue,
            };

            // 在source数据中查询
            let data_query = Query::select()
                .from(StringIden(music_list_one.t.clone()))
                .column(Asterisk)
                .and_where(Expr::col(StringIden(music_list_one.k)).eq(music_list_one.v))
                .clone();

            let (data_sql, data_values) = build_sqlx_query(data_query).await?;

            if let Ok(data_result) = sqlx::query_with(&data_sql, data_values)
                .fetch_one(&mut *conn)
                .await
            {
                match music_list_one.t.as_str() {
                    kuwo::KUWO => {
                        if let Ok(music) = KuwoMusic::from_row(data_result, id) {
                            music_indexs.push((music, index_));
                        }
                    }
                    _ => return Err(anyhow::anyhow!("Not supported source")),
                }
            }
        }
        // 根据index对music_indexs进行排序
        music_indexs.sort_by(|a, b| a.1.cmp(&b.1));

        // 提取并返回Music对象的Vec
        let musics = music_indexs
            .into_iter()
            .map(|(music, _)| music)
            .collect::<Vec<Music>>();

        Ok(musics)
    }

    // 更改自定义歌单中音乐的默认使用的源
    pub async fn change_music_default_source(
        &self,
        music_list: &MusicList,
        ids: Vec<i64>,
        default_sources: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        if ids.len() != default_sources.len() {
            return Err(anyhow::Error::msg(
                "Index and default_sources length mismatch",
            ));
        }

        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;

        for (id, default_source) in ids.into_iter().zip(default_sources.into_iter()) {
            let update_query = Query::update()
                .table(StringIden(music_list.name.to_string()))
                .value(DEFAULTSOURCE, default_source)
                .and_where(Expr::col(ID).eq(id))
                .to_owned();

            let (update_sql, update_values) = build_sqlx_query(update_query).await?;
            sqlx::query_with(&update_sql, update_values)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 对音乐数据的操作
    //// 只允许对音乐数据进行获取,修改,清理未被引用的音乐数据,不可手动删除

    // 读取所有音乐数据
    pub async fn read_music_data(&self, source: &str) -> Result<Vec<Music>, anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let query = Query::select()
            .from(StringIden(source.to_string()))
            .column(Asterisk)
            .clone();
        let (s, v) = build_sqlx_query(query).await?;
        let results = sqlx::query_with(&s, v).fetch_all(&mut *conn).await?;
        let mut musics: Vec<Music> = Vec::new();
        for result in results.into_iter() {
            match source {
                kuwo::KUWO => match KuwoMusic::from_row(result, 1) {
                    Ok(m) => musics.push(m),
                    Err(e) => {
                        println!("{e}")
                    }
                },
                _ => {
                    return Err(anyhow::anyhow!("Not Supported Source"));
                }
            }
        }
        Ok(musics)
    }

    // 更改音乐数据信息
    pub async fn change_music_data(
        &self,
        musics: &Vec<&Music>,
        infos: Vec<MusicInfo>,
    ) -> Result<(), anyhow::Error> {
        if musics.len() != infos.len() {
            return Err(anyhow::Error::msg("Musics and infos length mismatch"));
        }

        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;

        for (music, info) in musics.into_iter().zip(infos.into_iter()) {
            let query = music.to_sql_update(&info);
            match query {
                Ok(query) => {
                    let (update_sql, update_values) = build_sqlx_query(query).await?;
                    sqlx::query_with(&update_sql, update_values)
                        .execute(&mut *tx)
                        .await?;
                }
                Err(e) => {
                    println!("Failed to update: {e}");
                }
            }
        }

        tx.commit().await?;
        Ok(())
    }

    // 清理未被引用的音乐数据
    pub async fn clean_unused_music_data(&self) -> Result<(), anyhow::Error> {
        // 该函数由于目前只有KuWo，故仅考虑一种特点情况，后续需要随音乐源更新
        let mut conn = self.pool.acquire().await?;
        let mut tx = conn.begin().await?;

        // 获取所有KuWo音乐数据的ID
        let music_data_ids_query = Query::select()
            .from(StrIden(KUWO))
            .column(StrIden(&MUSIC_RID))
            .to_owned();
        let (music_data_ids_sql, music_data_ids_values) =
            build_sqlx_query(music_data_ids_query).await?;
        let music_data_music_rids_results =
            sqlx::query_with(&music_data_ids_sql, music_data_ids_values)
                .fetch_all(&mut *tx)
                .await?;

        // 获取所有自定义歌单
        let music_lists_query = Query::select()
            .from(REFMETADATA)
            .columns(vec![REFNAME])
            .clone();
        let (music_list_sql, music_list_values) = build_sqlx_query(music_lists_query).await?;

        let lists_results = sqlx::query_with(&music_list_sql, music_list_values)
            .fetch_all(&mut *tx)
            .await?;
        let mut list_names: Vec<String> = Vec::new();
        lists_results.into_iter().for_each(|r| {
            match r.try_get(REFNAME.0) {
                Ok(name) => {
                    list_names.push(name);
                }
                Err(_) => {}
            };
        });
        let mut used_ids: Vec<String> = Vec::new();
        for list_name in list_names.into_iter() {
            let refs_query = Query::select()
                .from(StringIden(list_name))
                .column(REFS)
                .to_owned();
            let (ref_sql, ref_values) = build_sqlx_query(refs_query).await?;
            let music_list_results = sqlx::query_with(&ref_sql, ref_values)
                .fetch_all(&mut *tx)
                .await?;
            for result in music_list_results {
                let refs: String = result.try_get(REFS.0)?;
                let music_list_data: Vec<Ref> = serde_json::from_str(&refs)?;
                for music_list_ in music_list_data {
                    if music_list_.t == KUWO {
                        used_ids.push(music_list_.v);
                    }
                }
            }
        }

        // 删除未被引用的音乐数据
        for result in music_data_music_rids_results {
            let rid: String = result.try_get(MUSIC_RID)?;
            if !used_ids.contains(&rid) {
                let delete_query = Query::delete()
                    .from_table(StrIden(KUWO))
                    .and_where(Expr::col(StrIden(MUSIC_RID)).eq(rid))
                    .to_owned();
                let (delete_sql, delete_values) = build_sqlx_query(delete_query).await?;
                sqlx::query_with(&delete_sql, delete_values)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// 数据库操作
    // 数据储存初始化创建表
    pub async fn init_create_table(&self) -> Result<(), anyhow::Error> {
        self.create_music_data_table().await?;
        self.create_music_list_metadata_table().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Instant};

    use crate::{KuwoSearch, SearchTrait};

    use super::*;
    use sqlx::any::install_default_drivers;
    use tokio::{fs::File, io::AsyncWriteExt};

    async fn setup_db() -> AnyPool {
        install_default_drivers();
        let path = PathBuf::from("_data/test.db");
        // if path.exists() {
        //     remove_file(path.clone()).await.unwrap();
        // }
        if !path.exists() {
            let _ = File::create(path).await.unwrap().shutdown().await;
        };
        let database_url = format!("sqlite:_data/test.db");
        AnyPool::connect(&database_url)
            .await
            .expect("Failed to connect to the database")
    }

    #[tokio::test]
    async fn test_init_table() {
        let pool = setup_db().await;
        let factory = SqlMusicFactory::new(pool);
        factory.init_create_table().await.unwrap();
    }

    #[tokio::test]
    async fn test_create_music_list() {
        let pool = setup_db().await;
        let test_music_list: MusicList = MusicList {
            name: "test_ref".to_string(),
            art_pic: "".to_string(),
            desc: "for test".to_string(),
        };
        let factory = SqlMusicFactory::new(pool);
        factory
            .create_music_list_table(&vec![test_music_list])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_insert_ref() {
        let pool = setup_db().await;
        let musics: Vec<Music> = KuwoSearch {}.search("张惠妹", 3, 30).await.unwrap();
        musics.iter().for_each(|m| {
            println!("{}", m.get_music_info());
        });
        let test_music_list: MusicList = MusicList {
            name: "test_ref".to_string(),
            art_pic: "".to_string(),
            desc: "for test".to_string(),
        };
        let mut musics_: Vec<&Music> = Vec::new();
        for music in musics.iter() {
            musics_.push(&music)
        }
        let factory = SqlMusicFactory::new(pool);
        factory
            .insert_music(&test_music_list, &musics_)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_del_music_index() {
        let pool = setup_db().await;
        let test_music_list: MusicList = MusicList {
            name: "test_ref".to_string(),
            art_pic: "".to_string(),
            desc: "for test".to_string(),
        };
        let factory = SqlMusicFactory::new(pool);
        factory
            .del_music(&test_music_list, (&[1, 3, 4]).to_vec())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_clean() {
        let pool = setup_db().await;
        let factory = SqlMusicFactory::new(pool);
        factory.clean_unused_music_data().await.unwrap();
    }

    #[tokio::test]
    async fn test_del_music_list() {
        let pool = setup_db().await;
        let test_music_list: MusicList = MusicList {
            name: "test_ref".to_string(),
            art_pic: "".to_string(),
            desc: "for test".to_string(),
        };
        let factory = SqlMusicFactory::new(pool);
        factory
            .del_music_list_table(&vec![test_music_list])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_change_music_list() {
        let pool = setup_db().await;
        let test_music_list: MusicList = MusicList {
            name: "test_ref".to_string(),
            art_pic: "".to_string(),
            desc: "for test".to_string(),
        };
        let new_test_music_list: MusicList = MusicList {
            name: "test_ref1".to_string(),
            art_pic: "".to_string(),
            desc: "".to_string(),
        };
        let factory = SqlMusicFactory::new(pool);
        factory
            .change_music_list_metadata(&vec![test_music_list], &vec![new_test_music_list])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_ref() {
        let start_time = Instant::now(); // 记录开始时间
        let pool = setup_db().await;
        let test_music_list: MusicList = MusicList {
            name: "test_ref".to_string(),
            art_pic: "".to_string(),
            desc: "for test".to_string(),
        };
        let factory = SqlMusicFactory::new(pool);
        let musics = factory.read_music(&test_music_list).await.unwrap();
        println!("{}", musics.len());
        musics.iter().for_each(|music| {
            let info = music.get_music_info();
            println!("{info}");
        });
        let elapsed_time = start_time.elapsed(); // 计算运行时间

        println!("代码运行时间: {:?}", elapsed_time);
    }

    #[tokio::test]
    async fn test_get_source() {
        let start_time = Instant::now(); // 记录开始时间
        let pool = setup_db().await;
        let name = kuwo::KUWO;
        let factory = SqlMusicFactory::new(pool);
        let musics = factory.read_music_data(name).await.unwrap();
        println!("{}", musics.len());
        musics.iter().for_each(|music| {
            let info = music.get_music_info();
            println!("{}", info);
            println!("{:?}", info.art_pic);
        });
        let elapsed_time = start_time.elapsed(); // 计算运行时间

        println!("代码运行时间: {:?}", elapsed_time);
    }

    #[tokio::test]
    async fn test_reorder() {
        let pool = setup_db().await;

        let test_music_list: MusicList = MusicList {
            name: "test_ref".to_string(),
            art_pic: "".to_string(),
            desc: "for test".to_string(),
        };
        let factory = SqlMusicFactory::new(pool);
        let musics: Vec<Music> = factory.read_music(&test_music_list).await.unwrap();
        musics
            .iter()
            .for_each(|m| println!("{}", m.get_music_info()));
        // 生成新的顺序索引
        let new_order: Vec<i64> = (0..musics.len() as i64).rev().collect();
        factory
            .reorder_music(&test_music_list, new_order, &musics)
            .await
            .unwrap();
        let musics: Vec<Music> = factory.read_music(&test_music_list).await.unwrap();
        musics
            .iter()
            .for_each(|m| println!("{}", m.get_music_info()));
    }
}
