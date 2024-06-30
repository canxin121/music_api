use lazy_static::lazy_static;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use sea_query::Iden;
use sea_query::{
    MysqlQueryBuilder, PostgresQueryBuilder, QueryBuilder, SchemaBuilder, SchemaStatementBuilder,
    SqliteQueryBuilder,
};
use sea_query_binder::{SqlxBinder, SqlxValues};
use std::sync::Arc;
use tokio::sync::RwLock;
lazy_static! {
    pub static ref CLIENT: ClientWithMiddleware = ClientBuilder::new(
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
    )
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(3),
    ))
    .build();
    pub static ref QUERY_BUILDER: Arc<RwLock<Box<dyn QueryBuilder + Send + Sync>>> = {
        let builder: Box<dyn QueryBuilder + Send + Sync> = Box::new(SqliteQueryBuilder);
        Arc::new(RwLock::new(builder))
    };
    pub static ref SCHEMA_BUILDER: Arc<RwLock<Box<dyn SchemaBuilder + Send + Sync>>> = {
        let builder: Box<dyn SchemaBuilder + Send + Sync> = Box::new(SqliteQueryBuilder);
        Arc::new(RwLock::new(builder))
    };
}

pub enum BuilderEnum {
    MySQL,
    SQLite,
    Postgres,
}

pub async fn init_builder(builder: BuilderEnum) -> Result<(), anyhow::Error> {
    let query_builder_: Box<dyn QueryBuilder + Send + Sync> = match builder {
        BuilderEnum::MySQL => Box::new(MysqlQueryBuilder),
        BuilderEnum::SQLite => Box::new(SqliteQueryBuilder),
        BuilderEnum::Postgres => Box::new(PostgresQueryBuilder),
    };
    let mut global_builder = QUERY_BUILDER.write().await;
    (*global_builder) = query_builder_;
    let chema_builer: Box<dyn SchemaBuilder + Send + Sync> = match builder {
        BuilderEnum::MySQL => Box::new(MysqlQueryBuilder),
        BuilderEnum::SQLite => Box::new(SqliteQueryBuilder),
        BuilderEnum::Postgres => Box::new(PostgresQueryBuilder),
    };
    let mut global_builder = SCHEMA_BUILDER.write().await;
    (*global_builder) = chema_builer;
    Ok(())
}

pub struct StrIden(pub &'static str);
impl Iden for StrIden {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).unwrap();
    }
}

pub struct StringIden(pub String);
impl Iden for StringIden {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).unwrap();
    }
}

pub async fn build_sqlx_query<Q: SqlxBinder>(
    query: Q,
) -> Result<(String, SqlxValues), anyhow::Error> {
    let builder = QUERY_BUILDER.read().await;
    Ok(query.build_any_sqlx(builder.as_ref()))
}

pub async fn build_query<T: SchemaStatementBuilder>(query: T) -> Result<String, anyhow::Error> {
    let builder = SCHEMA_BUILDER.read().await;
    Ok(query.build_any(builder.as_ref()))
}
