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
) -> Result<(), Box<dyn Error>> {
    let mut transaction: postgres::Transaction<'_> = client.transaction()?;

    // insert file first
    let row = transaction.query_one(
        "INSERT INTO files(file_name, extension) VALUES($1, $2) RETURNING file_id;",
        &[&file_result.filename, &file_result.file_extention],
    )?;
    let file_id: i32 = row.get(0);

    // insert all chunks for this file
    // use "COPY" as much faster
    let mut writer =
        transaction.copy_in("COPY chunks (content, embeddings, file_id) FROM STDIN")?;

    for (i, f) in file_result.chunks.iter().enumerate() {
        writeln!(writer, "{}\t{:?}\t{}", f.content, f.embedding, file_id)?;
    }
    writer.finish()?;
    transaction.commit()?;

    Ok(())
}
