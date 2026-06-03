use pdfsink_rs::PdfDocument;
use std::{
    error::Error,
    ffi::OsString,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

const VALID_FILE_EXTENSIONS: [&str; 2] = ["pdf", "txt"];

#[derive(Debug)]
pub struct FileDetail {
    pub path: String,
    pub extension: String,
}

pub fn get_files(dir: &Path, files: &mut Vec<FileDetail>) -> Result<(), Box<dyn Error>> {
    // this function will read all files in a directory
    // including files within all nested directories

    // it will only return files that are "accepted" file extensions

    if dir.is_dir() {
        for e in fs::read_dir(dir)? {
            let entry: DirEntry = e?;
            let path: PathBuf = entry.path();

            // recursive loop
            // read nested dir
            if path.is_dir() {
                get_files(&path.as_path(), files)?;
                continue;
            }
            if path.extension().is_none() {
                continue;
            }

            let ext = path.extension().unwrap();
            if ext.to_str().is_none() {
                continue;
            }
            let ext_str = ext.to_str().unwrap();
            let path_str = path.to_string_lossy().into_owned();

            files.push(FileDetail {
                path: path_str,
                extension: ext_str.to_string(),
            });
        }

        return Ok(());
    }

    // single file
    let path_str: String = dir.to_string_lossy().into_owned();
    if dir.extension().is_none() {
        return Ok(()); // early return
    }

    let ext = dir.extension().unwrap();
    if ext.to_str().is_none() {
        return Ok(()); // early return
    }
    let ext_str = ext.to_str().unwrap();

    files.push(FileDetail {
        path: path_str,
        extension: ext_str.to_string(),
    });

    Ok(())
}

pub fn is_valid_file_extension(file: &FileDetail) -> bool {
    if VALID_FILE_EXTENSIONS.contains(&file.extension.as_str()) {
        return true;
    }
    return false;
}

pub fn parse_txt(file: &FileDetail) -> Result<Vec<String>, Box<dyn Error>> {
    // get the raw string as a text
    let file_text = fs::read_to_string(&file.path)?;

    // split text by whitespace
    // and space to get individual words
    let mut words: Vec<String> = vec![];
    for word in file_text.trim().split(" ") {
        words.push(word.to_string());
    }

    return Ok(words);
}
