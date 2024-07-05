use sea_query::{Asterisk, Cond, Expr, InsertStatement, Query};
use sqlx::{Acquire as _, Row as _};

use crate::{
    music_aggregator::{music_aggregator_sql::SqlMusicAggregator, MusicAggregator},
    platform_integrator::{
        kuwo::{self, kuwo_music::KuwoMusic},
        wangyi::{self, WyMusic},
        ALL,
    },
    util::{build_sqlx_query, StringIden},
    Music, MusicInfo, MusicListInfo,
};

use super::{ObjectUnsafeStore as _, Ref, SqlFactory, DEFAULTSOURCE, ID, INDEX, POOL, REFS};

macro_rules! acquire_conn {
    () => {{
        let pool_lock = POOL.lock().await;
        let pool = pool_lock.as_ref().unwrap();
        pool.acquire().await?
    }};
}

impl SqlFactory {
    // 向自定义歌单插入音乐
    pub async fn add_musics(
        music_list_name: &str,
        musics: &Vec<&MusicAggregator>,
    ) -> Result<(), anyhow::Error> {
        // 将所有的音乐都插入到对应的音乐数据表中
        // 这里不会因为重复插入而触发Error
        for aggregator in musics {
            SqlFactory::insert_music_data(&aggregator.get_all_musics()).await?
        }

        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;

        // 获取最大ID,作为新插入时的Index(指示顺序)，这样可以确保新增的顺序在最后
        let result = sqlx::query(&format!("SELECT MAX(Id) FROM '{}'", music_list_name))
            .fetch_one(&mut *tx)
            .await?;
        let max_id: i64 = result.try_get(0).unwrap_or(0);

        // 插入所有的引用
        let indexs: Vec<i64> = (max_id + 1..max_id + 1 + musics.len() as i64).collect();

        for (aggregator, index) in musics.iter().zip(indexs) {
            let refs: Vec<Ref> = aggregator
                .get_all_musics()
                .iter()
                .map(|m| {
                    let (k, v) = m.get_primary_kv();
                    Ref {
                        t: m.source().to_owned(),
                        k,
                        v,
                    }
                })
                .collect();

            let query = InsertStatement::new()
                .into_table(StringIden(music_list_name.to_string()))
                .columns(vec![REFS, DEFAULTSOURCE, INDEX])
                .values_panic(vec![
                    serde_json::to_string(&refs).unwrap().into(),
                    aggregator.get_default_source().into(),
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
    pub async fn del_musics(music_list_name: &str, ids: Vec<i64>) -> Result<(), anyhow::Error> {
        let mut conn = acquire_conn!();
        let mut tx: sqlx::Transaction<'_, sqlx::Any> = conn.begin().await?;

        for id in ids {
            let delete_query = Query::delete()
                .from_table(StringIden(music_list_name.to_string()))
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

    // 更新自定义歌单中的音乐
    pub async fn replace_musics(
        music_list_name: &str,
        ids: Vec<i64>,
        musics: Vec<MusicAggregator>,
    ) -> Result<(), anyhow::Error> {
        if ids.len() != musics.len() {
            return Err(anyhow::Error::msg("Index and musics length mismatch"));
        }

        // 先插入所有的音乐数据
        // 这里不会因为重复插入而触发Error
        SqlFactory::insert_music_data(&musics.iter().flat_map(|m| m.get_all_musics()).collect())
            .await?;

        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;

        for (id, aggregator) in ids.into_iter().zip(musics.into_iter()) {
            let refs: Vec<Ref> = aggregator
                .get_all_musics()
                .iter()
                .map(|m| {
                    let (k, v) = m.get_primary_kv();
                    Ref {
                        t: m.source().to_owned(),
                        k,
                        v,
                    }
                })
                .collect();

            let query = Query::update()
                .table(StringIden(music_list_name.to_string()))
                .value(REFS, serde_json::to_string(&refs)?)
                .and_where(Expr::col(ID).eq(id))
                .to_owned();
            let (update_sql, update_values) = build_sqlx_query(query).await?;
            sqlx::query_with(&update_sql, update_values)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // 更改自定义歌单的歌曲顺序
    pub async fn reorder_musics(
        music_list_name: &str,
        new_full_index: &[i64],
        full_ids_in_order: &[i64],
    ) -> Result<(), anyhow::Error> {
        // ID=>主键(歌曲在自定义歌单中的主键), INDEX=>顺序
        // 传入的ids_in_order是 按照index从小到大排序好的主键
        // 传入的new_index是对应位置的新的index值(这个取的值不是表中真实的index值中的,而是认为ids_in_order的index是从1开始连续排的)
        // 比如会认为ids_in_order[a,b,c,d,e]的对应的index是[1,2,3,4,5],而不是表中的真实index值
        // 使用索引重排法来实现, 因此使用时必须一次性更新所有行的数据

        if new_full_index.len() != full_ids_in_order.len() {
            return Err(anyhow::Error::msg("Index and musics length mismatch"));
        }

        let mut conn = acquire_conn!();

        let mut tx = conn.begin().await?;

        let result = sqlx::query(&format!("SELECT COUNT(*) FROM '{}'", music_list_name))
            .fetch_one(&mut *tx)
            .await?;
        let max_index: i64 = result.try_get(0).unwrap_or(0);
        if (max_index as usize) != full_ids_in_order.len() {
            return Err(anyhow::Error::msg("Index and musics not enough"));
        }

        for (new_index, old_id) in new_full_index
            .into_iter()
            .zip(full_ids_in_order.into_iter())
        {
            let update_query = Query::update()
                .table(StringIden(music_list_name.to_string()))
                .value(INDEX, *new_index)
                .and_where(Expr::col(ID).eq(*old_id))
                .to_owned();

            let (update_sql, update_values) = build_sqlx_query(update_query).await?;

            sqlx::query_with(&update_sql, update_values)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    // 获取自定义歌单中的所有音乐(只带默认源)
    pub async fn get_all_musics(
        musiclist_info: &MusicListInfo,
    ) -> Result<Vec<MusicAggregator>, anyhow::Error> {
        let mut conn = acquire_conn!();
        let music_list_query = Query::select()
            .from(StringIden(musiclist_info.name.to_string()))
            .columns(vec![ID, INDEX, DEFAULTSOURCE, REFS])
            .clone();
        let (sql, values) = build_sqlx_query(music_list_query).await?;
        let results = sqlx::query_with(&sql, values).fetch_all(&mut *conn).await?;

        let mut music_indexs = Vec::new();

        for row in results.iter() {
            let (index, default_source, refs, id): (i64, String, String, i64) = (
                row.try_get(INDEX.0)?,
                row.try_get(DEFAULTSOURCE.0)?,
                row.try_get(REFS.0)?,
                row.try_get(ID.0)?,
            );
            let music_refs: Vec<Ref> = serde_json::from_str(&refs)?;
            let sources = music_refs.iter().map(|r| r.t.clone()).collect::<Vec<_>>();

            if let Some(selected) = music_refs.iter().find(|r| r.t == default_source) {
                let data_query = Query::select()
                    .from(StringIden(selected.t.clone()))
                    .column(Asterisk)
                    .and_where(Expr::col(StringIden(selected.k.clone())).eq(selected.v.clone()))
                    .clone();
                let (data_sql, data_values) = build_sqlx_query(data_query).await?;

                if let Ok(data_result) = sqlx::query_with(&data_sql, data_values)
                    .fetch_one(&mut *conn)
                    .await
                {
                    let music_aggregator = match selected.t.as_str() {
                        kuwo::KUWO => KuwoMusic::from_row(data_result, id).map(|music| {
                            SqlMusicAggregator::from_musics(
                                id,
                                musiclist_info.clone(),
                                default_source.clone(),
                                sources.clone(),
                                vec![music],
                            )
                        }),
                        wangyi::WANGYI => WyMusic::from_row(data_result, id).map(|music| {
                            SqlMusicAggregator::from_musics(
                                id,
                                musiclist_info.clone(),
                                default_source.clone(),
                                sources.clone(),
                                vec![music],
                            )
                        }),
                        _ => Err(anyhow::anyhow!("Not supported source")),
                    }?;
                    music_indexs.push((music_aggregator, index));
                }
            }
        }

        music_indexs.sort_by_key(|&(_, index)| index);
        Ok(music_indexs
            .into_iter()
            .map(|(music, _)| Box::new(music) as MusicAggregator)
            .collect())
    }

    // 获取自定义歌单中的指定音乐的指定源,将使用第一个源作为默认
    pub async fn get_music_by_id(
        music_list_info: &MusicListInfo,
        id: i64,
        sources: &[&str],
    ) -> Result<MusicAggregator, anyhow::Error> {
        // 判断source是否为空
        if sources.is_empty() {
            return Err(anyhow::anyhow!("sources can't be empty"));
        }
        let mut conn = acquire_conn!();
        let query = Query::select()
            .from(StringIden(music_list_info.name.clone()))
            .column(REFS)
            .and_where(Expr::col(ID).eq(id))
            .to_owned();
        let (sql, values) = build_sqlx_query(query).await?;

        let result = sqlx::query_with(&sql, values).fetch_one(&mut *conn).await?;
        let refs = serde_json::from_str::<Vec<Ref>>(&result.try_get::<String, _>(REFS.0)?)?;

        let mut musics = Vec::new();

        if sources.contains(&ALL) {
            for ref_ in refs.iter() {
                let query = Query::select()
                    .from(StringIden(ref_.t.clone()))
                    .column(Asterisk)
                    .and_where(Expr::col(StringIden(ref_.k.clone())).eq(ref_.v.clone()))
                    .to_owned();
                let (sql, values) = build_sqlx_query(query).await?;
                let result = sqlx::query_with(&sql, values).fetch_one(&mut *conn).await?;
                match ref_.t.as_str() {
                    kuwo::KUWO => musics.push(KuwoMusic::from_row(result, id)?),
                    wangyi::WANGYI => musics.push(WyMusic::from_row(result, id)?),
                    _ => return Err(anyhow::anyhow!("Not supported source")),
                }
            }
        } else {
            for source in sources.iter() {
                if let Some(ref_) = refs.iter().find(|r| &r.t == *source) {
                    let query = Query::select()
                        .from(StringIden(ref_.t.clone()))
                        .column(Asterisk)
                        .and_where(Expr::col(StringIden(ref_.k.clone())).eq(ref_.v.clone()))
                        .to_owned();
                    let (sql, values) = build_sqlx_query(query).await?;
                    let result = sqlx::query_with(&sql, values).fetch_one(&mut *conn).await?;
                    match ref_.t.as_str() {
                        kuwo::KUWO => musics.push(KuwoMusic::from_row(result, id)?),
                        wangyi::WANGYI => musics.push(WyMusic::from_row(result, id)?),
                        _ => return Err(anyhow::anyhow!("Not supported source")),
                    }
                }
            }
        }
        if musics.is_empty() {
            return Err(anyhow::anyhow!("Music source not find"));
        }
        let music_aggregator = SqlMusicAggregator::from_musics(
            id,
            music_list_info.clone(),
            sources.first().unwrap().to_string(),
            refs.iter().map(|r| r.t.clone()).collect(),
            musics,
        );
        Ok(Box::new(music_aggregator))
    }

    // 更改自定义歌单中音乐的默认使用的源
    pub async fn change_music_default_source(
        music_list_name: &str,
        ids: Vec<i64>,
        new_default_sources: Vec<String>,
    ) -> Result<(), anyhow::Error> {
        if ids.len() != new_default_sources.len() {
            return Err(anyhow::Error::msg(
                "Index and default_sources length mismatch",
            ));
        }

        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;

        for (id, default_source) in ids.into_iter().zip(new_default_sources.into_iter()) {
            let update_query = Query::update()
                .table(StringIden(music_list_name.to_string()))
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

    // 更改音乐数据信息
    pub async fn change_music_info(
        musics: &[Music],
        new_infos: Vec<MusicInfo>,
    ) -> Result<(), anyhow::Error> {
        if musics.len() != new_infos.len() {
            return Err(anyhow::Error::msg("Musics and infos length mismatch"));
        }

        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;

        for (music, info) in musics.into_iter().zip(new_infos.into_iter()) {
            let query = music.to_sql_update(&info).await;
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
}

#[cfg(test)]
mod test {
    use crate::{
        factory::{online_factory::aggregator_search, sql_factory::SqlFactory},
        filter::MusicFuzzFilter,
        platform_integrator::{kuwo::KUWO, wangyi::WANGYI, ALL},
        MusicListInfo,
    };
    // 写一个宏来初始化测试环境，包括初始化一个测试数据库(名字通过传入来指定)
    // 然后创建一个歌单，搜索一些音乐，然后插入音乐
    // 最后返回这个歌单和这个AggregatorOnlineFactory
    macro_rules! init_test_env {
        ($db_name:expr,$source:expr) => {{
            let path = format!("./_data/{}", $db_name);
            // 如果已存在，先删除
            if std::path::Path::new(&path).exists() {
                std::fs::remove_file(&path).unwrap();
            }
            SqlFactory::init_from_path(&path).await.unwrap();
            let mut aggregator_search = aggregator_search::AggregatorOnlineFactory::new();
            aggregator_search
                .search_music_aggregator(
                    $source,
                    "张国荣",
                    1,
                    30,
                    Some(&MusicFuzzFilter {
                        name: None,
                        artist: vec!["张国荣".to_string()],
                        album: None,
                    }),
                )
                .await
                .unwrap();
            aggregator_search.aggregators.iter().for_each(|aggregator| {
                println!("{}", aggregator);
            });

            let musiclist_info = MusicListInfo {
                id: 0,
                name: "歌单1".to_string(),
                art_pic: "".to_string(),
                desc: "".to_string(),
                extra: None,
            };
            // 创建歌单
            SqlFactory::create_musiclist(&vec![musiclist_info.clone()])
                .await
                .unwrap();

            // 插入音乐
            SqlFactory::add_musics(
                &musiclist_info.name,
                &aggregator_search.get_aggregator_refs(),
            )
            .await
            .unwrap();
            (musiclist_info, aggregator_search)
        }};
    }
    // 测试替换音乐
    #[tokio::test]
    async fn sql_factory_music_action_test_replace_music() {
        let (musiclist_info, _aggregator_search) = init_test_env!(
            "sql_factory_music_action_test_get_music.db",
            &[KUWO.to_string()]
        );
        let mut musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        for music in musics.iter_mut() {
            let _ = music.fetch_musics(vec![WANGYI.to_string()]).await;
        }
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        musics
            .iter()
            .find(|a| a.get_available_sources().contains(&WANGYI.to_string()))
            .unwrap();
        let first = musics.first().unwrap();
        first.fetch_lyric().await.unwrap();
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let first = musics.first().unwrap();
        assert!(first.get_default_music().get_music_info().lyric.is_some());
    }
    // 测试获取music
    #[tokio::test]
    async fn sql_factory_music_action_test_get_music() {
        let (musiclist_info, _aggregator_search) = init_test_env!(
            "sql_factory_music_action_test_get_music.db",
            &[ALL.to_string()]
        );
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let first = musics.first().unwrap();
        let id = first.get_music_id();
        let _aggregator_ = SqlFactory::get_music_by_id(&musiclist_info, id, &[KUWO])
            .await
            .unwrap();
    }

    // 测试插入和删除
    #[tokio::test]
    async fn sql_factory_music_action_test1() {
        let (musiclist_info, aggregator_search) =
            init_test_env!("sql_factory_music_action_test1.db", &[ALL.to_string()]);
        // 通过长度检验是否插入成功
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        assert_eq!(musics.len(), aggregator_search.aggregators.len());
        // 删除一部分
        let ids = musics.iter().map(|m| m.get_music_id()).collect::<Vec<_>>();
        SqlFactory::del_musics(&musiclist_info.name, ids[0..2].to_vec())
            .await
            .unwrap();
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        assert_eq!(musics.len(), aggregator_search.aggregators.len() - 2);
    }

    // 测试排序
    #[tokio::test]
    async fn sql_factory_music_action_test2() {
        let (musiclist_info, _) =
            init_test_env!("sql_factory_music_action_test2.db", &[ALL.to_string()]);
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();

        let musics_ids = musics.iter().map(|m| m.get_music_id()).collect::<Vec<_>>();
        let musics_enumrate_indexs = musics_ids
            .iter()
            .enumerate()
            .map(|(i, _)| i as i64)
            .collect::<Vec<_>>();
        let new_index = musics_enumrate_indexs
            .iter()
            .rev()
            .map(|i| *i)
            .collect::<Vec<_>>();

        SqlFactory::reorder_musics(&musiclist_info.name, &new_index, &musics_ids)
            .await
            .unwrap();
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();

        let new_music_ids = musics.iter().map(|m| m.get_music_id()).collect::<Vec<_>>();

        assert_eq!(
            musics_ids,
            new_music_ids.iter().rev().cloned().collect::<Vec<_>>()
        );
    }
    // 测试修改信息
    #[tokio::test]
    async fn sql_factory_music_action_change_info() {
        let (musiclist_info, _) = init_test_env!(
            "sql_factory_music_action_test_change_info.db",
            &[ALL.to_string()]
        );
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let first = musics.first().unwrap();
        first.fetch_lyric().await.unwrap();
        let mut new_info = first.get_default_music().get_music_info();
        new_info.name = "测试修改".to_string();
        new_info.artist = vec!["测试修改".to_string()];
        SqlFactory::change_music_info(&[first.get_default_music().clone()], [new_info].to_vec())
            .await
            .unwrap();
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let first = musics.first().unwrap();
        let info = first.get_default_music().get_music_info();
        assert!(info.lyric.is_some());
        assert!(info.name == "测试修改");
        assert!(info.artist == vec!["测试修改".to_string()]);
    }
    #[tokio::test]
    async fn sql_factory_music_action_test_change_music_default_source() {
        let (musiclist_info, _aggregator_search) = init_test_env!(
            "sql_factory_music_action_test_change_default_source.db",
            &[ALL.to_string()]
        );
        let musics = SqlFactory::get_all_musics(&musiclist_info).await.unwrap();
        let ids = musics
            .iter()
            .filter(|m| m.get_available_sources().contains(&WANGYI.to_string()))
            .map(|m| m.get_music_id())
            .collect::<Vec<_>>();
        let new_default_sources = vec![WANGYI.to_string(); ids.len()];

        SqlFactory::change_music_default_source(
            &musiclist_info.name,
            ids.clone(),
            new_default_sources.clone(),
        )
        .await
        .unwrap();

        for id in ids {
            let music_aggregator = SqlFactory::get_music_by_id(&musiclist_info, id, &[WANGYI])
                .await
                .unwrap();
            assert_eq!(music_aggregator.get_default_source(), WANGYI.to_string());
        }
    }
}
