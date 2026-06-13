mod cli;
mod db;
mod utils;

use clap::Parser;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::{
    error::Error,
    time::{Duration, Instant},
};

use indicatif::{ProgressBar, ProgressStyle};

use cli::Args;
use utils::FileDetail;

use postgres::{Client, NoTls};

use crate::{
    cli::Commands,
    db::postgres::copy_chunks,
    utils::{
        DatabaseType, FilesChunkResults, VALID_FILE_EXTENSIONS, chunk_text, embed_chunks,
        extract_text_from_file, get_files,
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();

    match args.command {
        Commands::List {} => {
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

        Commands::Chunk {
            path,
            exts,
            model,
            size,
            database_url,
        } => {
            let cli_config: cli::CliChunkConfig =
                Commands::get_cli_chunk_config(path, exts, model, size, &database_url)?;

            let files: Vec<FileDetail> = get_files(
                &cli_config.path_to_parse.as_path(),
                &cli_config.exts_to_parse,
            )?;

            if files.len() == 0 {
                println!(
                    "there are no files to embed. valid files extensions: {}",
                    VALID_FILE_EXTENSIONS.join(", ")
                );
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

            let start = Instant::now();

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
            if cli_config.database_type == DatabaseType::Postgres {
                let mut client = Client::connect(&database_url, NoTls)?;
                copy_chunks(&mut client, &file_results, cli_config.model_to_use)?;
            } else if cli_config.database_type == DatabaseType::Mysql {
            }

            pb.finish_and_clear();

            let num_of_chunks = {
                let mut i = 0;
                for f in file_results {
                    i += f.chunks.len()
                }
                i
            };

            println!(
                "\n=======================
Files Parsed : {}
Chunks Created: {}
Elapsed Time : {:.2?}
=======================",
                files.len(),
                num_of_chunks,
                start.elapsed(),
            );

            return Ok(());
        }
    }
}
