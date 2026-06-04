mod utils;

use std::{
    error::Error,
    ffi::{OsStr, OsString},
    time::Instant,
};

use utils::FileDetail;

use crate::utils::{chunk_text_file, get_files, is_valid_file_extension};
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

    for f in &valid_files {
        if f.extension == "txt" {
            let chunk_results = match chunk_text_file(f) {
                Ok(x) => x,
                Err(x) => return Err(x),
            };
        }
    }

    return Ok(());
}
