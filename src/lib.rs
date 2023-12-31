use eyre::Result;
use sqlx::{pool::PoolOptions, Pool, Postgres};

pub mod authors;
pub mod books;

/// Complete this function so that it connects to a Postgres instance and returns the pool.
///
/// For testing purposes the PgPoolOptions have already been created for you. All you need to do is configure the connection if you want/need and connect to the database.
pub async fn connect(pool_options: PoolOptions<Postgres>) -> Result<Pool<Postgres>> {
    let database_url =
        std::env::var("DATABASE_URL").expect("Missing environment variable DATATBASE_URL");

    let pool = pool_options.connect(&database_url).await?;

    Ok(pool)
}
