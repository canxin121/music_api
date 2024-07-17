use sea_query::{ColumnDef, Cond, Expr, InsertStatement, Query, Table, TableCreateStatement};
use sqlx::{Acquire as _, Row as _};

use crate::{
    music_aggregator::music_aggregator_online::merge_music_aggregators,
    music_list::MusicList,
    util::{build_query, build_sqlx_query, StrIden, StringIden},
    MusicAggregator, MusicListInfo,
};

use super::{
    sql_musiclist::SqlMusicList, SqlFactory, DEFAULTSOURCE, ID, INDEX, POOL, REFARTPIC, REFDESC,
    REFMETADATA, REFNAME, REFS,
};
macro_rules! acquire_conn {
    () => {{
        let pool_lock = POOL.lock().await;
        let pool = pool_lock.as_ref().unwrap();
        pool.acquire().await?
    }};
}
impl SqlFactory {
    // 内部方法，删除自定义歌单的元数据
    pub(crate) async fn del_music_list_metadata(
        musiclist_names: &[&str],
    ) -> Result<(), anyhow::Error> {
        let mut cond = Cond::any();
        for r in musiclist_names {
            cond = cond.add(Expr::col(REFNAME).eq(r.to_string()));
        }
        let query = Query::delete()
            .from_table(REFMETADATA)
            .cond_where(cond)
            .to_owned();
        let mut conn = acquire_conn!();
        let mut tx: sqlx::Transaction<'_, sqlx::Any> = conn.begin().await?;

        let (delete_sql, delete_values) = build_sqlx_query(query).await?;
        sqlx::query_with(&delete_sql, delete_values)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    // 内部方法，插入自定义歌单的元数据
    pub(crate) async fn insert_music_list_matadata(
        music_lists: &Vec<MusicListInfo>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;

        // 获取最大ID,作为新插入时的Index(指示顺序)，这样可以确保新增的顺序在最后
        let result = sqlx::query(&format!("SELECT MAX(Id) FROM '{}'", REFMETADATA.0))
            .fetch_one(&mut *tx)
            .await?;
        let max_id: i64 = result.try_get(0).unwrap_or(0);

        for music_list in music_lists {
            let query = InsertStatement::new()
                .into_table(REFMETADATA)
                .columns(vec![REFNAME, REFDESC, REFARTPIC, INDEX])
                .values_panic(vec![
                    music_list.name.clone().into(),
                    music_list.desc.clone().into(),
                    music_list.art_pic.clone().into(),
                    (max_id + 1).into(),
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

    // 创建自定义歌单表
    pub async fn create_musiclist(
        music_list_infos: &Vec<MusicListInfo>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;
        for music_list in music_list_infos {
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
        SqlFactory::insert_music_list_matadata(music_list_infos).await?;
        Ok(())
    }

    // 修改自定义歌单信息
    pub async fn change_musiclist_info(
        old: &Vec<MusicListInfo>,
        new: &Vec<MusicListInfo>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = acquire_conn!();
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
            if new_one.art_pic != old_one.art_pic {
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
                        query.and_where(Expr::col(ID).eq(old_one.id));
                        let (s, q) = build_sqlx_query(query).await?;
                        sqlx::query_with(&s, q).execute(&mut *tx).await?;
                    }
                    Err(e) => {
                        println!("Failed to rename table:{e}")
                    }
                };
            } else {
                query.and_where(Expr::col(ID).eq(old_one.id));
                let (s, q) = build_sqlx_query(query).await?;
                sqlx::query_with(&s, q).execute(&mut *tx).await?;
            }
        }
        tx.commit().await?;
        Ok(())
    }

    // 获取所有自定义歌单
    pub async fn get_all_musiclists() -> Result<Vec<MusicList>, anyhow::Error> {
        let mut conn = acquire_conn!();
        // 先获取所有的ref数据
        let music_list_query = Query::select()
            .from(REFMETADATA)
            .columns(vec![REFNAME, REFARTPIC, REFDESC, INDEX, ID])
            .clone();

        let (music_list_sql, music_list_values) = build_sqlx_query(music_list_query).await?;

        let music_list_results = sqlx::query_with(&music_list_sql, music_list_values)
            .fetch_all(&mut *conn)
            .await?;

        let mut results: Vec<(MusicList, i64)> = Vec::new();
        for result in music_list_results {
            let index: i64 = result.try_get(INDEX.0)?;
            if let Ok(musiclist_info) = MusicListInfo::from_row(result) {
                let musiclist = Box::new(SqlMusicList::new(musiclist_info)) as MusicList;
                results.push((musiclist, index));
            }
        }
        results.sort_by_key(|&(_, index)| index);
        Ok(results
            .into_iter()
            .map(|(musiclist, _)| musiclist)
            .collect())
    }

    // 重新排序歌单
    pub async fn reorder_musiclist(new_ids: &[i64]) -> Result<(), anyhow::Error> {
        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;

        let result = sqlx::query(&format!("SELECT COUNT(*) FROM '{}'", REFMETADATA.0))
            .fetch_one(&mut *tx)
            .await?;
        let real_length: i64 = result.try_get(0)?;
        if (real_length as usize) != new_ids.len() {
            return Err(anyhow::anyhow!("Indexs and musics has wrong length"));
        }

        for (index, id) in new_ids.into_iter().enumerate() {
            let update_query = Query::update()
                .table(REFMETADATA)
                .value(INDEX, (index as i64) + 1)
                .and_where(Expr::col(ID).eq(id.to_string()))
                .to_owned();
            let (update_sql, update_values) = build_sqlx_query(update_query).await?;
            sqlx::query_with(&update_sql, update_values)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // 删除一个自定义歌单
    pub async fn del_musiclist(musiclist_names: &[&str]) -> Result<(), anyhow::Error> {
        // 先删去元数据
        SqlFactory::del_music_list_metadata(musiclist_names).await?;
        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;
        for r in musiclist_names {
            let query = Table::drop()
                .table(StringIden(r.to_string()))
                .if_exists()
                .to_owned();
            let s = build_query(query).await?;
            sqlx::query(&s).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // 删除歌单中的重复音乐
    pub async fn del_duplicate_musics_of_musiclist(
        musiclist_info: &MusicListInfo,
    ) -> Result<(), anyhow::Error> {
        let aggs = SqlFactory::get_all_musics(musiclist_info).await?;
        let merged_aggs = merge_music_aggregators(aggs).await?;
        SqlFactory::del_musiclist(&[musiclist_info.name.as_str()]).await?;
        SqlFactory::create_musiclist(&vec![musiclist_info.clone()]).await?;
        SqlFactory::add_musics(&musiclist_info.name, &merged_aggs.iter().collect()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::factory::sql_factory::SqlFactory;
    use crate::music_list::{ExtraInfo, MusicListInfo};
    use tokio::test;

    #[test]
    async fn complex_musiclist_test() {
        // 初始化数据库连接
        SqlFactory::init_from_path("./_data/test_musiclist_complex_action.db")
            .await
            .unwrap();

        // 1. 创建初始歌单
        let initial_music_list = vec![MusicListInfo {
            id: 0,
            name: "test".to_string(),
            desc: "test description".to_string(),
            art_pic: "test_art_pic".to_string(),
            extra: Some(ExtraInfo {
                play_count: Some(100),
                music_count: Some(10),
            }),
        }];
        SqlFactory::create_musiclist(&initial_music_list)
            .await
            .unwrap();

        // 2. 验证歌单创建
        let all_music_lists = SqlFactory::get_all_musiclists().await.unwrap();
        assert_eq!(all_music_lists.len(), 1);
        assert_eq!(all_music_lists[0].get_musiclist_info().name, "test");
        assert_eq!(
            all_music_lists[0].get_musiclist_info().desc,
            "test description"
        );
        assert_eq!(
            all_music_lists[0].get_musiclist_info().art_pic,
            "test_art_pic"
        );

        // 3. 修改歌单信息
        let modified_music_list = vec![MusicListInfo {
            id: 0,
            name: "test_modified".to_string(),
            desc: "modified description".to_string(),
            art_pic: "modified_art_pic".to_string(),
            extra: None,
        }];
        SqlFactory::change_musiclist_info(&initial_music_list, &modified_music_list)
            .await
            .unwrap();

        // 验证修改后的歌单信息
        let all_music_lists = SqlFactory::get_all_musiclists().await.unwrap();
        assert_eq!(all_music_lists.len(), 1);
        assert_eq!(
            all_music_lists[0].get_musiclist_info().name,
            "test_modified"
        );
        assert_eq!(
            all_music_lists[0].get_musiclist_info().desc,
            "modified description"
        );
        assert_eq!(
            all_music_lists[0].get_musiclist_info().art_pic,
            "modified_art_pic"
        );

        // 4. 批量创建多个歌单
        let multiple_music_lists = vec![
            MusicListInfo {
                id: 0,
                name: "playlist1".to_string(),
                desc: "playlist1 description".to_string(),
                art_pic: "playlist1_art_pic".to_string(),
                extra: None,
            },
            MusicListInfo {
                id: 0,
                name: "playlist2".to_string(),
                desc: "playlist2 description".to_string(),
                art_pic: "playlist2_art_pic".to_string(),
                extra: None,
            },
        ];
        SqlFactory::create_musiclist(&multiple_music_lists)
            .await
            .unwrap();

        // 验证所有歌单
        let all_music_lists = SqlFactory::get_all_musiclists().await.unwrap();
        assert_eq!(all_music_lists.len(), 3); // 包括之前的修改后的歌单
        let names: Vec<_> = all_music_lists
            .iter()
            .map(|ml| ml.get_musiclist_info().name.clone())
            .collect();
        assert!(names.contains(&"test_modified".to_string()));
        assert!(names.contains(&"playlist1".to_string()));
        assert!(names.contains(&"playlist2".to_string()));

        // 5. 删除歌单
        SqlFactory::del_musiclist(&vec!["playlist1", "playlist2"])
            .await
            .unwrap();
        let all_music_lists = SqlFactory::get_all_musiclists().await.unwrap();
        assert_eq!(all_music_lists.len(), 1);
        assert_eq!(
            all_music_lists[0].get_musiclist_info().name,
            "test_modified"
        );

        // 6. 清理环境
        SqlFactory::del_musiclist(&vec!["test_modified"])
            .await
            .unwrap();
        let all_music_lists = SqlFactory::get_all_musiclists().await.unwrap();
        assert_eq!(all_music_lists.len(), 0);
    }
}
