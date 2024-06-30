use sea_query::{Asterisk, Expr, Query};
use sqlx::{Acquire, Row as _};

use crate::{
    platform_integrator::{
        kuwo::{
            self,
            kuwo_music::{KuwoMusic, MUSIC_RID},
            KUWO,
        },
        wangyi::{self, WyMusic},
    },
    util::{build_sqlx_query, StrIden, StringIden},
    Music,
};

use super::{ObjectUnsafeStore, Ref, SqlFactory, POOL, REFMETADATA, REFNAME, REFS};
macro_rules! acquire_conn {
    () => {{
        let pool_lock = POOL.lock().await;
        let pool = pool_lock.as_ref().unwrap();
        pool.acquire().await?
    }};
}

impl SqlFactory {
    // 插入音乐数据
    pub(crate) async fn insert_music_data(musics: &Vec<&Music>) -> Result<(), anyhow::Error> {
        let mut conn = acquire_conn!();
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

    // 清理未被引用的音乐数据
    pub async fn clean_unused_music_data() -> Result<(), anyhow::Error> {
        // 该函数由于目前只有KuWo，故仅考虑一种特点情况，后续需要随音乐源更新
        let mut conn = acquire_conn!();
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

    // id是音乐在一个歌单中的主键
    // 读取所有音乐数据，由于这里是直接读取数据库，没有经过歌单，故id也无意义
    pub async fn read_music_data(source: &str) -> Result<Vec<Music>, anyhow::Error> {
        let mut conn = acquire_conn!();
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
                wangyi::WANGYI => match WyMusic::from_row(result, 1) {
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
}
