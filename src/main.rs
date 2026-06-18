mod cli;
mod constants;
mod db;
mod utils;

use clap::Parser;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use mysql::{Opts, Pool, prelude::Queryable};
use std::{
    error::Error,
    time::{Duration, Instant},
};

use db::{DatabaseType::{Mysql, Postgres}};

use indicatif::{ProgressBar, ProgressStyle};

use cli::Args;
use utils::FileDetail;

use postgres::{Client, NoTls};

use crate::{
    cli::{Commands, ListCommands},
    constants::{DOCUMENT_EXTENSIONS, TEXT_EXTENSIONS},
    db::{
        mysql::{copy_chunks_mysql, new_mysql_client},
        postgres::{copy_chunks_postgres, new_postgres_client},
    },
    utils::{
        FilesChunkResults, chunk_text, embed_chunks, extract_text_from_file, get_files,
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    let mut valid_file_extensions: Vec<&str> = [TEXT_EXTENSIONS, DOCUMENT_EXTENSIONS].concat();

    match args.command {
        Commands::List { command } => match command {
            ListCommands::Models => {
                struct ModelList {
                    str_name: String,
                    model: EmbeddingModel,
                }

                let mut models: Vec<ModelList> = TextEmbedding::list_supported_models()
                    .iter()
                    .map(|x| ModelList {
                        str_name: x.model_code.clone(),
                        model: x.model.clone(),
                    })
                    .collect();
                models.sort_by_key(|x| x.str_name.clone());

                println!("Supported models:");
                for m in models {
                    println!("{:?} | ({:?})", m.model, m.str_name)
                }
                return Ok(());
            }

            ListCommands::Files => {
                valid_file_extensions.sort();
                println!("Supported file extensions:\n{:#?}", valid_file_extensions);
                Ok(())
            }
        },

        Commands::Chunk {
            path,
            exts,
            model,
            size,
            database_url,
            require_ssl,
        } => {
            let cli_config: cli::CliChunkConfig = Commands::get_cli_chunk_config(
                path,
                exts,
                model,
                size,
                &database_url,
                &valid_file_extensions,
            )?;

            let mut files: Vec<FileDetail> = get_files(
                &cli_config.path_to_parse.as_path(),
                &cli_config.exts_to_parse,
            )?;

            // filter out any files that dont belong to a supported file extension
            // modifies existing files vector in-place
            files.retain(|x| valid_file_extensions.contains(&x.extension.as_str()));

            if files.len() == 0 {
                println!("no files to embed");
                return Ok(());
            }

            // create the embedding model once outside of all the chunking logic
            // so doesnt have to be recreated constantly
            let mut embedding_model: TextEmbedding =
                TextEmbedding::try_new(InitOptions::new(EmbeddingModel::BGESmallENV15))?;

            let mut file_results: Vec<FilesChunkResults> = vec![];

            let pb: ProgressBar = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::with_template("{spinner} {msg}")?);
            pb.enable_steady_tick(Duration::from_millis(100));

            let start: Instant = Instant::now();

            for (i, f) in files.iter().enumerate() {
                pb.set_message(format!(
                    "File {:?}/{:?} | {:?}",
                    i + 1,
                    files.len(),
                    f.filename
                ));

                // 1. extract text from files
                let file_text: String = extract_text_from_file(&f)?;

                // 2. extract chunks from text
                let mut chunks: Vec<utils::Chunk> = chunk_text(&file_text, &cli_config.chunk_size);

                // 3. embed each chunk and set the embedding field on the struct
                embed_chunks(&mut chunks, &mut embedding_model)?;

                file_results.push(FilesChunkResults {
                    filename: f.filename.clone(),
                    file_extention: f.extension.clone(),
                    chunks,
                });
            }

            // insert chunks into database
            pb.set_message("inserting chunks into database");
            match cli_config.database_type {
                Postgres => {
                    let mut client = new_postgres_client(require_ssl, &database_url)?;
                    copy_chunks_postgres(&mut client, &file_results, cli_config.model_to_use)?;
                }
                Mysql => {
                    let mut conn = new_mysql_client(require_ssl, &database_url)?;

                    // mysql version must be at least v9 to support vector columns
                    let mysql_version: Option<String> = conn.query_first("SELECT VERSION();")?;
                    if let Some(x) = mysql_version {
                        if !x.starts_with("9") {
                            return Err(format!(
                                "vector embeddings support requires MySQL version 9.0+"
                            )
                            .into());
                        }
                    } else {
                        return Err(format!("could not determine MySQL version").into());
                    }

                    match copy_chunks_mysql(&mut conn, &file_results, cli_config.model_to_use) {
                        Ok(()) => (),
                        Err(e) => {
                            // if an error happens with mysql, need to manually delete the tables
                            // created bc in mysql the CREATE TABLE statements are implcit commits,
                            // transaction rollback will not remove the tables

                            conn.query_drop("DROP TABLE IF EXISTS chunks;").ok();
                            conn.query_drop("DROP TABLE IF EXISTS files;").ok();
                            return Err(e);
                        }
                    }
                }
            }

            pb.finish_and_clear();

            println!(
                "\n=======================
Files Parsed : {}
Chunks Created: {}
Elapsed Time : {:.2?}
=======================",
                files.len(),
                {
                    let mut i = 0;
                    for f in file_results {
                        i += f.chunks.len()
                    }
                    i
                },
                start.elapsed(),
            );

            return Ok(());
        }
    }
}
