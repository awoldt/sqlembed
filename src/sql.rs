/*
    the whole point of this cli tool is to generate sql that you can
    quickly run against ur local sql database

    will include 2 main table
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

use crate::utils::Chunk;

pub fn generate_sql(chunks: &Vec<FilesChunkResults>, dimensionality: i32) -> String {
    // generate sql create tables query
    let mut str: String = String::new();
    str.push_str(
        format!(
            "
        CREATE TABLE files(
            file_id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
            absolute_path TEXT NOT NULL,
            extension VARCHAR(250) NOT NULL
        );

        CREATE TABLE chunks(
            chunk_id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
            content VARCHAR(255) NOT NULL,
            embeddings VECTOR({}) NOT NULL,
            file_id INT,
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
                "INSERT INTO files(file_id, absolute_path, extension) VALUES({},'{}','{}');\n",
                file_id, chunk.filename, chunk.file_extention
            )
            .as_str(),
        );

        // now add all the chunks for each file
        for c in &chunk.chunks {
            str.push_str(
                format!(
                    "INSERT INTO chunks(content, embeddings, file_id) VALUES('{}',{:?},{});\n",
                    c.content.replace("'", "''"),
                    c.embedding,
                    file_id
                )
                .as_str(),
            )
        }
    }

    return str.to_owned();
}

pub fn write_sql_to_filesystem(query: &str) -> Result<(), Box<dyn Error>> {
    // this function will take the final sql queries generated above and
    // write to a single ".sql" file in the cwd

    std::fs::write("sql.sql", query)?;

    Ok(())
}
