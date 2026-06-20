use std::error::Error;

use ::mysql::{PooledConn, prelude::Queryable};
use ::postgres::Client;
use fastembed::{EmbeddingModel, ModelInfo};

use crate::db::DatabaseType::{Mysql, Postgres};

pub mod mysql;
pub mod postgres;

#[derive(PartialEq)]
pub enum DatabaseType {
    Postgres,
    Mysql,
}

/*
    creates the 2 tables before all the chunk
    insertions occur works for any database type
*/
pub fn create_tables(
    database_type: &DatabaseType,
    postgres_client: Option<&mut Client>,
    mysql_client: Option<&mut PooledConn>,
    embedding_model: &ModelInfo<EmbeddingModel>,
) -> Result<(), Box<dyn Error>> {
    match database_type {
        Postgres => match postgres_client {
            Some(client) => {
                client.query(
                    &format!(
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
                    ),
                    &[],
                )?;
            }

            None => {
                return Err(format!("error while creating tables").into());
            }
        },
        Mysql => match mysql_client {
            Some(client) => {
                client.query_drop(
                    "
                        CREATE TABLE files (
                            file_id INT AUTO_INCREMENT PRIMARY KEY,
                            file_name TEXT NOT NULL,
                            extension VARCHAR(250) NOT NULL
                        );
    ",
                )?;

                client.query_drop(format!(
                    "CREATE TABLE chunks (
                chunk_id INT AUTO_INCREMENT PRIMARY KEY,
                content LONGTEXT NOT NULL,
                embeddings VECTOR({}) NOT NULL,
                file_id INT NOT NULL,
                FOREIGN KEY (file_id) REFERENCES files(file_id) ON DELETE CASCADE
            );",
                    embedding_model.dim
                ))?;
            }

            None => {
                return Err(format!("error while creating tables").into());
            }
        },
    }

    Ok(())
}
