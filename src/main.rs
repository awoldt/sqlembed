mod cli;
mod sql;
mod utils;

use clap::Parser;
use core::time;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::{
    error::Error,
    ffi::{OsStr, OsString},
    time::{Duration, Instant},
};

use indicatif::{ProgressBar, ProgressStyle};

use cli::Args;
use utils::FileDetail;

use crate::{
    cli::get_cli_config,
    sql::{FilesChunkResults, generate_sql, write_sql_to_filesystem},
    utils::{
        EmbeddingModelUsed::BGESmallENV15, VALID_FILE_EXTENSIONS, chunk_text, embed_chunks,
        extract_text_from_file, get_files,
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    let cli_config: cli::CliConfig = get_cli_config(&args)?;

    let pb: ProgressBar = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} {msg}")?);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message("gathering files for chunking");

    let files: Vec<FileDetail> = get_files(
        &cli_config.path_to_parse.as_path(),
        &cli_config.exts_to_parse,
    )?;

    if files.len() == 0 {
        pb.finish_and_clear();
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

    let start = Instant::now();

    for f in &files {
        pb.set_message(format!("chunking {:?}", f.filename));

        // 1. extract text from files
        let file_text = extract_text_from_file(&f)?;

        // 2. extract chunks from text
        let mut chunks = chunk_text(&file_text);

        // 3. embed each chunk and set the embedding field on the struct
        embed_chunks(&mut chunks, &mut embedding_model)?;

        file_results.push(FilesChunkResults {
            filename: f.filename.clone(),
            file_extention: f.extension.clone(),
            chunks,
        });
    }

    pb.finish_and_clear();

    let sql_string = generate_sql(&file_results, BGESmallENV15)?;
    write_sql_to_filesystem(&sql_string)?;

    let num_of_chunks = {
        let mut i = 0;
        for f in file_results {
            i += f.chunks.len()
        }
        i
    };

    println!(
        "\n\n=======================
Successfully parsed {} files and generated {} chunks in {:.2?} seconds
    ",
        files.len(),
        num_of_chunks,
        start.elapsed()
    );

    return Ok(());
}
