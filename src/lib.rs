pub mod data;
mod server;

use std::sync::{Arc, LazyLock};

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

pub static CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| {
    ClientBuilder::new(
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap(),
    )
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(2),
    ))
    .build()
});

static DB_POOL: LazyLock<Arc<RwLock<Option<DatabaseConnection>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));
