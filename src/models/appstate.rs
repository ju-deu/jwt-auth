use std::sync::Arc;
use sqlx::{Pool, Postgres};

pub struct Appstate {
    pub db_pool: Arc<Pool<Postgres>>,
    pub jwt_secret: String,
}