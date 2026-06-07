/*
    the whole point of this cli tool is to generate sql that you can
    quickly run against ur local sql database

    will include 2 main tables
    - chunks
    - files

    tons and tons of chunk files. each chunk has an association with a file
*/

use std::error::Error;

pub struct FilesChunkResults {
    pub filename: String,
    pub file_extention: String,
    pub chunks: Vec<Chunk>,
}

use fastembed::EmbeddingModel;

use crate::utils::{Chunk};

pub fn generate_sql(
    chunks: &Vec<FilesChunkResults>,
    embedding_model: EmbeddingModel,
) -> Result<String, Box<dyn Error>> {
    // the dimensions of the vectors depend on the model used to embed
    let dimensionality;

    match embedding_model {
        EmbeddingModel::BGESmallENV15 => dimensionality = 384,
        _ => {
            return Err("must use a valid embedding model
             "
            .into());
        }
    }

    // generate sql create tables query
    let mut str: String = String::new();
    str.push_str(
        format!(
            "
        CREATE TABLE files(
            file_id INT PRIMARY KEY,
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
            dimensionality
        )
        .as_str(),
    );

    str.push_str("\n");

    // now add all the files
    for (i, chunk) in chunks.iter().enumerate() {
        let file_id = i + 1;

        str.push_str(
            format!(
                "INSERT INTO files(file_id, file_name, extension) VALUES({},'{}','{}');\n",
                file_id, chunk.filename, chunk.file_extention
            )
            .as_str(),
        );

        // now add all the chunks for each file
        for c in &chunk.chunks {
            str.push_str(
                format!(
                    "INSERT INTO chunks(content, embeddings, file_id) VALUES('{}','{:?}',{});\n",
                    c.content.replace("'", "''"),
                    c.embedding,
                    file_id
                )
                .as_str(),
            )
        }
    }

    return Ok(str.to_owned());
}

pub fn write_sql_to_filesystem(query: &str) -> Result<(), Box<dyn Error>> {
    // this function will take the final sql queries generated above and
    // write to a single ".sql" file in the cwd

    std::fs::write("sql.sql", query)?;

    Ok(())
}
