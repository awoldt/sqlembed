mod utils;

use std::{
    error::Error,
    ffi::{OsStr, OsString},
};

use utils::FileDetail;

use crate::utils::{chunk, embed_text, get_files, is_valid_file_extension, parse_txt};
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

    for f in valid_files {
        if f.extension == "txt".to_string() {
            let text = match parse_txt(&f) {
                Ok(x) => x,
                Err(e) => return Err(format!("{:#?}\nerror while parsing text file", e).into()),
            };
            let chunks = chunk(text);

            let embedded_text = embed_text(&chunks)?;
            println!("{:?}", embedded_text);
        }
    }

    return Ok(());
}
