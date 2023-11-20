use eyre::Result;
use sqlx::{Pool, Postgres};

/// Fill out the run function so that it will seed the database with the data defined in the [README]("../../README.md")
pub async fn run(pool: Pool<Postgres>) -> Result<()> {
    let mut transaction = pool.begin().await?;
    let mut is_ok = true;

    if let Err(error) = sqlx::query_file!("./sql/authors.sql")
        .execute(&mut *transaction)
        .await
    {
        eprintln!("Error seeding authors: {error}");
        is_ok = false;
    }

    if let Err(error) = sqlx::query_file!("./sql/books.sql")
        .execute(&mut *transaction)
        .await
    {
        eprintln!("Error seeding books: {error}");
        is_ok = false;
    }

    if let Err(error) = sqlx::query_file!("./sql/book_authors.sql")
        .execute(&mut *transaction)
        .await
    {
        eprintln!("Error seeding book_authors: {error}");
        is_ok = false;
    }

    if is_ok {
        transaction.commit().await?;
    } else {
        eprintln!("Reverting seeds");
        transaction.rollback().await?;
    }

    Ok(())
}
