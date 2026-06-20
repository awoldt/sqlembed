use fastembed::{EmbeddingModel, ModelInfo};
use native_tls::TlsConnector;
use postgres::{Client, NoTls};
use postgres_native_tls::MakeTlsConnector;
use std::error::Error;
use std::io::Write;

use crate::parse::FilesChunkResults;

pub fn new_postgres_client(
    require_ssl: bool,
    database_url: &str,
) -> Result<Client, Box<dyn Error>> {
    if !require_ssl {
        let client: Client = Client::connect(&database_url, NoTls)?;
        return Ok(client);
    }

    let connector: TlsConnector = TlsConnector::builder().build()?;
    let connector: MakeTlsConnector = MakeTlsConnector::new(connector);

    let client: Client = Client::connect(&database_url, connector)?;

    Ok(client)
}

pub fn insert_chunk_postgres(
    client: &mut Client,
    file_result: &FilesChunkResults,
    file_index: &mut i32,
) -> Result<(), Box<dyn Error>> {
    let mut transaction: postgres::Transaction<'_> = client.transaction()?;

    // insert file first
    transaction.query(
        "INSERT INTO files(file_name, extension) VALUES($1, $2);",
        &[&file_result.filename, &file_result.file_extention],
    )?;

    // insert all chunks for this file
    for c in &file_result.chunks {
        transaction.query(
            "INSERT INTO chunks (content, embeddings, file_id) VALUES($1, $2, $3);",
            &[&c.content, &c.embedding, file_index],
        )?;
    }

    transaction.commit()?;

    *file_index += 1;

    Ok(())
}
