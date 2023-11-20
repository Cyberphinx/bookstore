use crate::authors::{Author, AuthorId, BookAuthors};
use eyre::Result;
use sqlx::{query, query_as, Pool, Postgres};
use std::collections::HashMap;

/// Id for the books in the database
pub type BookId = i32;

/// Insert a book with the given name into the database and return the newly created book id
pub async fn create_book(pool: &Pool<Postgres>, name: &str) -> Result<BookId> {
    let book_id = sqlx::query!(
        "INSERT INTO books (name) VALUES ($1) RETURNING book_id;",
        name
    )
    .fetch_one(pool)
    .await?
    .book_id;

    Ok(book_id)
}

/// Retrieve a single book from the database and return it. We don't care about the authors of this book yet.
///
/// In the case that the provided id doesn't exist in the database, return a None
pub async fn get_book_by_id(pool: &Pool<Postgres>, book_id: BookId) -> Result<Option<Book>> {
    match sqlx::query_as!(Book, "SELECT * FROM books WHERE book_id = $1 ;", book_id)
        .fetch_one(pool)
        .await
    {
        Ok(book) => Ok(Some(book)),
        Err(_) => Ok(None),
    }
}

/// Retrieve all of the books from the database and return them. We don't care about the authors yet, so this will just be a Vector of simple book objects
pub async fn get_all_books(pool: &Pool<Postgres>) -> Result<Vec<Book>> {
    let books = sqlx::query_as!(Book, "SELECT * FROM books;")
        .fetch_all(pool)
        .await?;
    Ok(books)
}

/// Update the books name with the given id in the database.
pub async fn update_book(pool: &Pool<Postgres>, name: &str, book_id: BookId) -> Result<()> {
    sqlx::query!(
        "UPDATE books SET name = $1 WHERE book_id = $2;",
        name,
        book_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Permanently delete the book with the given id from the database
pub async fn delete_book(pool: &Pool<Postgres>, book_id: BookId) -> Result<()> {
    sqlx::query!("DELETE FROM books WHERE book_id = $1;", book_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Create a book and it's author at the same and associate them together in the database. Return a tuple with the book and author ids.
///
/// Since this is a bulk operation make sure that if any command fails during it the database is left unchanged.
pub async fn create_book_and_author(
    pool: &Pool<Postgres>,
    book_name: &str,
    author_name: &str,
) -> Result<(BookId, AuthorId)> {
    let mut transaction = pool.begin().await?;

    let book_id = match query!(
        "INSERT INTO books (name) VALUES ($1) RETURNING book_id;",
        book_name
    )
    .fetch_one(&mut *transaction)
    .await
    {
        Ok(book) => Some(book.book_id),
        Err(error) => {
            eprintln!("Problem inserting book: {}", error);
            None
        }
    };

    let author_id = match query!(
        "INSERT INTO authors (name) VALUES ($1) RETURNING author_id;",
        author_name
    )
    .fetch_one(&mut *transaction)
    .await
    {
        Ok(author) => Some(author.author_id),
        Err(error) => {
            eprintln!("Problem inserting author: {}", error);
            None
        }
    };

    if let (Some(book_id), Some(author_id)) = (book_id, author_id) {
        match query!(
            "INSERT INTO book_authors (author_id, book_id) VALUES ($1, $2);",
            author_id,
            book_id
        )
        .execute(&mut *transaction)
        .await
        {
            Ok(_) => {
                transaction.commit().await?;
                Ok((book_id, author_id))
            }
            Err(error) => {
                transaction.rollback().await?;
                eprintln!("{}", error);
                Err(eyre::eyre!(
                    "Error bulk inserting book and author, aborting"
                ))
            }
        }
    } else {
        transaction.rollback().await?;
        Err(eyre::eyre!(
            "Error bulk inserting book and author, aborting"
        ))
    }
}

/// The books that we're returing with the authors is a HashMap. Feel free to change this to any other kind of Map like a BTreeMap
pub type Books = HashMap<BookId, BookWithAuthors>;

/// Retrieve all books in the database with their authors and return using the Books type above.
///
/// Use a single operation to the database to get all of the data you need
pub async fn get_all_books_with_authors(pool: &Pool<Postgres>) -> Result<Books> {
    let book_authors = query_as!(
        BookAuthors,
        r#"
        SELECT 
            authors.author_id, 
            books.book_id, 
            authors.name as author_name, 
            books.name as book_name
        FROM book_authors 
            JOIN books ON books.book_id = book_authors.book_id 
            JOIN authors ON authors.author_id = book_authors.author_id;
        "#
    )
    .fetch_all(pool)
    .await?;
    dbg!(&book_authors);

    let mut book_with_authors = HashMap::new();
    for book_author in book_authors {
        let book = book_with_authors
            .entry(book_author.book_id)
            .or_insert(BookWithAuthors {
                book_id: book_author.book_id,
                name: book_author.book_name,
                authors: vec![],
            });
        let author = Author {
            author_id: book_author.author_id,
            name: book_author.author_name,
        };
        book.authors.push(author);
    }
    dbg!(&book_with_authors);

    Ok(book_with_authors)
}

/// This struct models just the books table. Use this struct when we don't care about the authors of the books
#[derive(Debug)]
pub struct Book {
    pub book_id: BookId,
    pub name: String,
}

/// This struct models the relationship between a book and it's authors. Use this struct when returning both books and their authors together
#[derive(Debug)]
pub struct BookWithAuthors {
    pub book_id: BookId,
    pub name: String,
    pub authors: Vec<Author>,
}
