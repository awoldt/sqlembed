mod utils;

use std::{
    error::Error,
    ffi::{OsStr, OsString},
    time::Instant,
};

use utils::FileDetail;

use crate::utils::{chunk, embed_text, get_files, is_valid_file_extension, parse_pdf, parse_txt};
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

    let start = Instant::now();
    for f in &valid_files {
        let mut words: Vec<String> = vec![];

        if f.extension == "txt".to_string() {
            words = match parse_txt(&f) {
                Ok(x) => x,
                Err(e) => return Err(format!("{:#?}\nerror while parsing text file", e).into()),
            };
        }

        if f.extension == "pdf".to_string() {
            words = match parse_pdf(&f.path) {
                Ok(x) => x,
                Err(e) => return Err(format!("{:#?}\nerror while parsing pdf file", e).into()),
            };
        }

        let chunks = chunk(words);
        let embedded_text = embed_text(&chunks)?;
    }

    println!(
        "\nFinished chunking {} files in {:.2?}",
        &valid_files.len(),
        start.elapsed()
    );
    return Ok(());
}
