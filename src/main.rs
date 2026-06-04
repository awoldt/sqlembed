mod sql;
mod utils;

use std::{
    error::Error,
    ffi::{OsStr, OsString},
    time::{Duration, Instant},
};

use indicatif::{ProgressBar, ProgressStyle};

use utils::FileDetail;

use crate::{
    sql::{FilesChunkResults, generate_sql, write_sql_to_filesystem},
    utils::{
        EmbeddingModelUsed::BGESmallENV15, chunk_pdf_file, chunk_text_file, get_files,
        is_valid_file_extension,
    },
};
fn main() -> Result<(), Box<dyn Error>> {
    let cwd = std::env::current_dir()?;
    let mut files: Vec<FileDetail> = vec![]; // this will have all files
    match get_files(cwd.as_path(), &mut files) {
        Ok(x) => x,
        Err(x) => return Err(x),
    };

    // only use valid files
    let mut valid_files: Vec<FileDetail> = vec![];
    for f in files {
        if !is_valid_file_extension(&f) {
            continue;
        }
        valid_files.push(f);
    }

    let mut file_results: Vec<FilesChunkResults> = vec![];

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} {msg}")?);

    for f in &valid_files {
        pb.set_message(format!("chunking {:?}", f.path));
        pb.enable_steady_tick(Duration::from_millis(100));

        if f.extension == "txt" {
            let chunk_results = match chunk_text_file(f) {
                Ok(x) => x,
                Err(x) => return Err(x),
            };
            file_results.push(FilesChunkResults {
                filename: f.path.to_string(),
                chunks: chunk_results,
                file_extention: f.extension.to_string(),
            });
        }

        if f.extension == "pdf" {
            let chunk_results = match chunk_pdf_file(f) {
                Ok(x) => x,
                Err(x) => return Err(x),
            };
            file_results.push(FilesChunkResults {
                filename: f.path.to_string(),
                chunks: chunk_results,
                file_extention: f.extension.to_string(),
            });
        }
    }

    pb.finish();

    let sql_string = generate_sql(&file_results, BGESmallENV15)?;
    write_sql_to_filesystem(&sql_string)?;

    return Ok(());
}
