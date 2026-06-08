/*
    the whole point of this cli tool is to generate sql that you can
    quickly run against ur local postgres database

    will include 2 main tables
    - chunks
    - files

    each chunk inserted will include the appropriate dimensions based on the 
    embedding model used
*/

use std::error::Error;

pub struct FilesChunkResults {
    pub filename: String,
    pub file_extention: String,
    pub chunks: Vec<Chunk>,
}

use fastembed::{EmbeddingModel, ModelInfo};

use crate::utils::Chunk;

pub fn generate_sql(
    chunks: &Vec<FilesChunkResults>,
    embedding_model: ModelInfo<EmbeddingModel>,
) -> Result<String, Box<dyn Error>> {
    // generate sql create tables query
    let mut str: String = String::new();
    str.push_str(
        format!(
            " -- text embedding model used: {} ({})

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
            embedding_model.model, embedding_model.model_code, embedding_model.dim
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
