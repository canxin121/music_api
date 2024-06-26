use std::path::PathBuf;

use sea_query::{ColumnDef, InsertStatement, Query, TableCreateStatement};
use sqlx::{any::install_default_drivers, Acquire as _, AnyPool, Row as _};
use tokio::{fs::File, io::AsyncWriteExt as _};

use crate::{
    platform_integrator::{kuwo::kuwo_music::KuwoMusic, wangyi::WyMusic},
    util::{build_query, build_sqlx_query},
};

use super::{
    ObjectUnsafeStore, SqlFactory, ID, METADATA, POOL, REFARTPIC, REFDESC, REFMETADATA, REFNAME,
    VERSION,
};

macro_rules! acquire_conn {
    () => {{
        let pool_lock = POOL.lock().await;
        let pool = pool_lock.as_ref().unwrap();
        pool.acquire().await?
    }};
}

impl SqlFactory {
    // 获取数据库版本
    async fn get_version() -> Result<i32, anyhow::Error> {
        let mut conn = acquire_conn!();
        let query = Query::select().from(METADATA).column(VERSION).to_owned();
        let (s, v) = build_sqlx_query(query).await?;
        let result = sqlx::query_with(&s, v)
            .fetch_one(&mut *conn)
            .await?
            .try_get(VERSION.0)?;
        Ok(result)
    }

    // 从文件路径创建一个SqlMusicFactory，并自动升级数据库
    pub async fn init_from_path(filepath: &str) -> Result<(), anyhow::Error> {
        install_default_drivers();
        let path = PathBuf::from(filepath);
        if !path.exists() {
            File::create(&path).await?.shutdown().await?;
        }
        let database_url = format!("sqlite:{}", filepath);
        let pool = AnyPool::connect(&database_url).await?;

        SqlFactory::init(pool).await?;
        Ok(())
    }

    // 创建一个SqlMusicFactory，并自动升级数据库
    pub async fn init(pool: AnyPool) -> Result<(), anyhow::Error> {
        install_default_drivers();
        {
            let mut global_pool = POOL.lock().await;
            *global_pool = Some(pool);
        }
        SqlFactory::upgrade().await?;
        Ok(())
    }

    /// 数据库操作
    // 数据储存初始化创建表
    async fn create_all_table() -> Result<(), anyhow::Error> {
        let _ = SqlFactory::create_music_data_table().await;
        let _ = SqlFactory::create_music_list_metadata_table().await;
        let _ = SqlFactory::create_metadata_table().await?;
        Ok(())
    }

    // 数据库升级,暂不不支持降级操作
    // 数据库基本不会有大的改动，最常见的是 添加新的音乐原始数据表
    // 版本变迁历史:
    // 无版本号(0): 初始版本
    // 1: 添加了数据库元数据表(MetaData表)和一个原始音乐数据表(WangYi表)
    async fn upgrade() -> Result<(), anyhow::Error> {
        let version = SqlFactory::get_version().await.unwrap_or(0);
        match version {
            // 0->1
            // init_create_table即可完成所有工作，
            // 其内部会逐个创建所有的表(忽视中间的创建错误(已存在则会出错))+初始化数据库版本为1
            0 => {
                SqlFactory::create_all_table().await?;
            }
            _ => {}
        }
        Ok(())
    }
    // 创建自定义歌单元数据表
    async fn create_music_list_metadata_table() -> Result<(), anyhow::Error> {
        let query = TableCreateStatement::new()
            .table(REFMETADATA)
            .col(ColumnDef::new(REFNAME).string().not_null())
            .col(ColumnDef::new(REFARTPIC).string().not_null())
            .col(ColumnDef::new(REFDESC).integer())
            .col(ColumnDef::new(ID).integer().primary_key().auto_increment())
            .clone();
        let mut conn = acquire_conn!();
        let s: String = build_query(query).await?;
        sqlx::query(&s).execute(&mut *conn).await?;
        Ok(())
    }

    // 创建包含数据库版本信息的元数据表，并设置版本为1
    async fn create_metadata_table() -> Result<(), anyhow::Error> {
        let query = TableCreateStatement::new()
            .table(METADATA)
            .col(ColumnDef::new(ID).integer().primary_key().auto_increment())
            .col(ColumnDef::new(VERSION).integer())
            .clone();
        let mut conn = acquire_conn!();
        let s: String = build_query(query).await?;
        sqlx::query(&s).execute(&mut *conn).await?;

        // 插入版本信息
        let query = InsertStatement::new()
            .into_table(METADATA)
            .columns(vec![VERSION])
            .values_panic(vec![1.into()])
            .to_owned();
        let (insert_sql, insert_values) = build_sqlx_query(query).await?;
        sqlx::query_with(&insert_sql, insert_values)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    #[allow(unused)]
    async fn change_version(version: i32) -> Result<(), anyhow::Error> {
        let mut conn = acquire_conn!();
        let query = Query::update()
            .table(METADATA)
            .value(VERSION, version)
            .to_owned();
        let (s, v) = build_sqlx_query(query).await?;
        sqlx::query_with(&s, v).execute(&mut *conn).await?;
        Ok(())
    }

    // 初始化操作，创建所有原始数据表
    async fn create_music_data_table() -> Result<(), anyhow::Error> {
        // 中间可能因为已存在表而失败，直接忽略，防止影响后续创建
        macro_rules! create_and_execute_table {
            ($tx:expr, $model:ty) => {{
                let query = <$model>::create_table_query();
                let sql = build_query(query).await?;
                let _ = sqlx::query(&sql).execute(&mut *$tx).await;
            }};
        }

        let mut conn = acquire_conn!();
        let mut tx = conn.begin().await?;

        create_and_execute_table!(tx, KuwoMusic);
        create_and_execute_table!(tx, WyMusic);

        tx.commit().await?;
        Ok(())
    }
}
