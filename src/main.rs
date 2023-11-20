use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Error loading .env");

    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&database_url)
    //     .await
    //     .expect("Error connecting to datatbase");

    bookstore::connect(PgPoolOptions::new().max_connections(5))
        .await
        .expect("Error running queries");
}
