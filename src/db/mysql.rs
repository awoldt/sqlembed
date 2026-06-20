use crate::parse::FilesChunkResults;
use fastembed::{EmbeddingModel, ModelInfo};
use mysql::Value::Float;
use mysql::{Opts, OptsBuilder, Pool, SslOpts, TxOpts};
use mysql::{PooledConn, prelude::Queryable};
use std::error::Error;

pub fn new_mysql_client(
    require_ssl: bool,
    database_url: &str,
) -> Result<PooledConn, Box<dyn Error>> {
    if require_ssl {
        let opts = Opts::from_url(database_url)?;
        let opts_buidler = OptsBuilder::from_opts(opts).ssl_opts(Some(SslOpts::default()));
        let pool = Pool::new(opts_buidler)?;
        let conn = pool.get_conn()?;

        return Ok(conn);
    }

    let opts = Opts::from_url(&database_url)?;
    let pool = Pool::new(opts)?;
    let conn = pool.get_conn()?;
    Ok(conn)
}

pub fn insert_chunk_mysql(
    conn: &mut PooledConn,
    file_result: &FilesChunkResults,
    file_index: &mut i32,
) -> Result<(), Box<dyn Error>> {
    let mut transaction = conn.start_transaction(TxOpts::default())?;

    // insert file first
    transaction.exec_drop(
        "INSERT INTO files (file_name, extension) VALUES (?, ?)",
        (&file_result.filename, &file_result.file_extention),
    )?;

    // insert all chunks for this file
    for c in &file_result.chunks {
        transaction.exec_drop(
            "INSERT INTO chunks (content, embeddings, file_id) VALUES (?, ?, ?)",
            (
                &c.content,
                format!(
                    "[{}]",
                    c.embedding
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                ),
                *file_index,
            ),
        )?;
    }

    transaction.commit()?;

    *file_index += 1;
    Ok(())
}
