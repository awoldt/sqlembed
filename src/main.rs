mod cli;
mod constants;
mod db;
mod parse;

use clap::Parser;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use mysql::{Opts, Pool, PooledConn, prelude::Queryable};
use std::{
    error::Error,
    time::{Duration, Instant},
};

use db::DatabaseType::{Mysql, Postgres};

use indicatif::{ProgressBar, ProgressStyle};

use cli::Args;

use postgres::{Client, NoTls};

use crate::{
    cli::{Commands, ListCommands},
    constants::{DOCUMENT_EXTENSIONS, TEXT_EXTENSIONS},
    db::{
        mysql::{insert_chunk_mysql, new_mysql_client},
        postgres::{insert_chunk_postgres, new_postgres_client},
    },
    parse::{
        FileDetail, FilesChunkResults, chunk_text, embed_chunks, extract_text_from_file, get_files,
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

            // create a single database connection
            let mut mysql_client: Option<PooledConn> = None;
            let mut postgres_client: Option<Client> = None;

            match cli_config.database_type {
                Postgres => {
                    let mut client = new_postgres_client(require_ssl, &database_url)?;
                    postgres_client = Some(client);
                }
                Mysql => {
                    let mut client = new_mysql_client(require_ssl, &database_url)?;
                    mysql_client = Some(client);
                }
            }

            // get all the files
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
            let mut embedding_model: TextEmbedding =
                TextEmbedding::try_new(InitOptions::new(EmbeddingModel::BGESmallENV15))?;

            let pb: ProgressBar = ProgressBar::new_spinner();
            pb.set_style(ProgressStyle::with_template("{spinner} {msg}")?);
            pb.enable_steady_tick(Duration::from_millis(100));

            // loop through all files
            let start: Instant = Instant::now();
            let mut file_index: i32 = 1; // used for primary key inserts
            let mut successes: i32 = 0;
            let mut errors: i32 = 0;
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
                let mut chunks: Vec<parse::Chunk> = chunk_text(&file_text, &cli_config.chunk_size);

                // 3. embed each chunk and set the embedding field on the struct
                embed_chunks(&mut chunks, &mut embedding_model)?;

                // 4. insert file and chunks into database
                let file_result: FilesChunkResults = FilesChunkResults {
                    filename: f.filename.clone(),
                    file_extention: f.extension.clone(),
                    chunks,
                };

                match cli_config.database_type {
                    Postgres => {
                        if postgres_client.is_none() {
                            return Err(format!("could not establish postgres client").into());
                        }

                        match insert_chunk_postgres(
                            &mut postgres_client.unwrap(),
                            &file_result,
                            &mut file_index,
                        ) {
                            Ok(()) => {
                                successes += 1;
                                continue;
                            }
                            Err((e)) => {
                                errors += 1;

                                continue;
                            }
                        }
                    }

                    Mysql => {
                        if mysql_client.is_none() {
                            return Err(format!("could not establish mysql client").into());
                        }

                        match insert_chunk_mysql(
                            &mut mysql_client.unwrap(),
                            &file_result,
                            &mut file_index,
                        ) {
                            Ok(()) => {
                                successes += 1;
                                continue;
                            }
                            Err((e)) => {
                                errors += 1;

                                continue;
                            }
                        }
                    }
                }
            }

            pb.finish_and_clear();

            println!(
                "\n=======================
Files Parsed : {}
Elapsed Time : {:.2?}

Errors: {}

Embedding model used: {}
=======================",
                successes,
                start.elapsed(),
                format!("{} errors", errors),
                cli_config.model_to_use.model
            );

            return Ok(());
        }
    }
}
