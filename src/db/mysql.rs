use crate::utils::FilesChunkResults;
use fastembed::{EmbeddingModel, ModelInfo};
use mysql::Value::Float;
use mysql::{Opts, OptsBuilder, Pool, SslOpts, TxOpts};
use mysql::{PooledConn, prelude::Queryable};
use std::error::Error;

pub fn new_mysql_client(require_ssl: bool, database_url: &str) -> Result<PooledConn, Box<dyn Error>> {
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

pub fn copy_chunks_mysql(
    conn: &mut PooledConn,
    chunks: &Vec<FilesChunkResults>,
    embedding_model: &ModelInfo<EmbeddingModel>,
) -> Result<(), Box<dyn Error>> {
    // use a transaction!
    let mut transaction = conn.start_transaction(TxOpts::default())?;

    transaction.query_drop(
        "
        CREATE TABLE files (
            file_id INT AUTO_INCREMENT PRIMARY KEY,
            file_name TEXT NOT NULL,
            extension VARCHAR(250) NOT NULL
        );
    ",
    )?;

    transaction.query_drop(format!(
        "CREATE TABLE chunks (
                chunk_id INT AUTO_INCREMENT PRIMARY KEY,
                content LONGTEXT NOT NULL,
                embeddings VECTOR({}) NOT NULL,
                file_id INT NOT NULL,
                FOREIGN KEY (file_id) REFERENCES files(file_id) ON DELETE CASCADE
            );",
        embedding_model.dim
    ))?;


    // insert files first
    transaction.exec_batch(
        "
        INSERT INTO files (file_name, extension) VALUES (?,?);
    ",
        chunks
            .iter()
            .map(|x| (x.filename.as_str(), x.file_extention.as_str())),
    )?;

    // insert chunks
    transaction.exec_batch(
        "
                    INSERT INTO chunks (content, embeddings, file_id) VALUES (?,STRING_TO_VECTOR(?),?);
            ",
        chunks.iter().enumerate().flat_map(|(i, f)| {
            f.chunks.iter().map(move |x| {
                (
                    x.content.clone(),
                    format!(
                        "[{}]",
                        x.embedding
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    i + 1,
                )
            })
        }),
    )?;

    transaction.commit()?;

    Ok(())
}
