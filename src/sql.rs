/*
    the whole point of this cli tool is to generate sql that you can
    quickly run against ur local postgres database

    will include 2 main tables
    - chunks
    - files

    each chunk inserted will include the appropriate dimensions based on the
    embedding model used
*/

use clap::{Parser, ValueEnum};
use mysql::{Opts, Pool};
use pgvector::Vector;
use postgres::{
    Client, NoTls,
    binary_copy::BinaryCopyInWriter,
    types::{Kind, Type},
};
use std::{error::Error, io::Write};

#[derive(PartialEq)]
pub enum DatabaseType {
    Postgres,
    Mysql,
}

pub struct FilesChunkResults {
    pub filename: String,
    pub file_extention: String,
    pub chunks: Vec<Chunk>,
}

use fastembed::{EmbeddingModel, ModelInfo};

use crate::utils::Chunk;

pub fn copy_chunks(
    client: &mut Client,
    chunks: &Vec<FilesChunkResults>,
    embedding_model: ModelInfo<EmbeddingModel>,
) -> Result<(), Box<dyn Error>> {
    // use a transaction!
    let mut transaction = client.transaction()?;

    // create the tables first
    transaction.batch_execute(&format!(
        "
                            CREATE TABLE files(
                            file_id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                            file_name TEXT NOT NULL,
                            extension VARCHAR(250) NOT NULL
                        );

                        CREATE EXTENSION IF NOT EXISTS vector;
                        CREATE TABLE chunks(
                            chunk_id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
                            content TEXT NOT NULL,
                            embeddings VECTOR({}) NOT NULL,
                            file_id INT NOT NULL,
                            FOREIGN KEY (file_id) REFERENCES files(file_id) ON DELETE CASCADE
                        );
    ",
        embedding_model.dim
    ))?;

    // insert files first
    let mut writer = transaction.copy_in("COPY files (file_name, extension) FROM STDIN")?;
    for f in chunks {
        writeln!(writer, "{}\t{}", f.filename, f.file_extention)?;
    }
    writer.finish()?;

    // insert chunks
    let mut writer =
        transaction.copy_in("COPY chunks (content, embeddings, file_id) FROM STDIN")?;
    for (i, f) in chunks.iter().enumerate() {
        for c in &f.chunks {
            writeln!(writer, "{}\t{:?}\t{}", c.content, c.embedding, i + 1)?;
        }
    }
    writer.finish()?;

    transaction.commit()?;
    Ok(())
}
