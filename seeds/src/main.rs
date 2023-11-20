use sqlx::postgres::PgPoolOptions;

/// Connect to the database and run the seeds function in the `lib.rs` file
#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Error loading .env");

    let database_url =
        std::env::var("DATABASE_URL").expect("Missing environment variable DATABASE_URL");

    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Error connecting to database");

    seeds::run(pool).await.expect("Error running seeds");
}
