/*
    steps:
    1. read file into single string
    2. process file text into chunks (250 default)
    3. embed each chunk
    4. insert into database

*/

use docx_lite::parse_document_from_path;
use fastembed::TextEmbedding;
use pdfsink_rs::PdfDocument;
use std::{
    error::Error,
    fs::{self, DirEntry, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use crate::constants::{DOCUMENT_EXTENSIONS, TEXT_EXTENSIONS};

#[derive(Debug)]
pub struct FileDetail {
    pub extension: String,
    pub absolute_path: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct Chunk {
    pub content: String,     // raw human readable text
    pub embedding: Vec<f32>, // the vectors that represents this chunk of text
}

pub struct FilesChunkResults {
    pub filename: String,
    pub file_extention: String,
    pub chunks: Vec<Chunk>,
}

pub fn get_files(
    dir: &Path,
    exts_to_parse: &Vec<String>,
) -> Result<Vec<FileDetail>, Box<dyn Error>> {
    // this function will get all the "valid" files it finds in
    // the directory user wants to parse from, also factoring
    // in the specific extention types the user wants

    let mut files: Vec<FileDetail> = vec![];

    if dir.is_dir() {
        // directory of files
        // loop through each
        for e in fs::read_dir(dir)? {
            let entry: DirEntry = e?;
            let path: PathBuf = entry.path();

            // recursive loop, read nested dir
            if path.is_dir() {
                let mut nested_files: Vec<FileDetail> = get_files(&path.as_path(), exts_to_parse)?;
                files.append(&mut nested_files);
                continue;
            }
            if path.extension().is_none() {
                continue;
            }

            let ext = path.extension().unwrap();
            if ext.to_str().is_none() {
                continue;
            }
            if path.file_name().is_none() {
                continue;
            }

            let filename = path.file_name().unwrap();
            let ext_str = ext.to_str().unwrap();
            let path_str = path.to_string_lossy().into_owned();

            // check to see if the user only wants certain file extensions parsed
            if exts_to_parse.len() > 0 {
                if !exts_to_parse.contains(&ext_str.to_string()) {
                    continue;
                }
            }

            files.push(FileDetail {
                absolute_path: path_str,
                extension: ext_str.to_string(),
                filename: filename.to_string_lossy().into_owned(),
            });
        }

        return Ok(files);
    } else {
        // single file
        let path_str: String = dir.to_string_lossy().into_owned();
        if dir.extension().is_none() {
            return Ok(vec![]); // early return
        }

        let ext = dir.extension().unwrap();
        if ext.to_str().is_none() {
            return Ok(vec![]); // early return
        }
        if ext.to_str().is_none() {
            return Ok(vec![]); // early return
        }

        // check to see if the user only wants certain file extensions parsed
        if exts_to_parse.len() > 0 {
            if !exts_to_parse.contains(&ext.to_str().unwrap().to_string()) {
                return Ok(vec![]); // early return
            }
        }

        if dir.file_name().is_none() {
            return Ok(vec![]); // early return
        }

        let filename = dir.file_name();
        if filename.is_none() {
            return Ok(vec![]); // early return
        }

        files.push(FileDetail {
            absolute_path: path_str,
            extension: ext.to_str().unwrap().to_string(),
            filename: filename.unwrap().to_string_lossy().into_owned(),
        });

        Ok(files)
    }
}

pub fn extract_text_from_file(file: &FileDetail) -> Result<String, Box<dyn Error>> {
    if TEXT_EXTENSIONS.contains(&file.extension.as_str()) {
        let f: File = File::open(&file.absolute_path)?;
        let mut buf_reader: BufReader<File> = BufReader::new(f);
        let mut str = String::new();
        buf_reader.read_to_string(&mut str)?;
        return Ok(str);
    }

    if DOCUMENT_EXTENSIONS.contains(&file.extension.as_str()) {
        match file.extension.as_str() {
            "pdf" => {
                let pdf = PdfDocument::open(file.absolute_path.clone())?;
                if pdf.is_empty() {
                    // no pages on pdf, just return success with no chunks
                    return Ok(String::new());
                }
                let str = pdf.extract_text();
                return Ok(str);
            }

            "docx" => {
                let doc: docx_lite::Document =
                    parse_document_from_path(file.absolute_path.clone())?;
                let str: String = doc.extract_text();
                return Ok(str);
            }

            "pptx" => {
                let ppt = rustypptx::parse_pptx(Path::new(&file.absolute_path))?;
                let str: String = ppt.to_markdown();
                return Ok(str);
            }

            _ => {
                return Err("not a valid file extension to chunk".into());
            }
        }
    }

    Err("not a valid file extension to chunk".into())
}

pub fn chunk_text(file_text: &str, chunk_size: &i32) -> Vec<Chunk> {
    // take the file text and splits into "chunks"

    let mut chunks: Vec<Chunk> = vec![];
    let mut chunk_text: Vec<String> = vec![];

    for word in file_text.split_whitespace() {
        if chunk_text.len() >= *chunk_size as usize {
            chunks.push(Chunk {
                content: chunk_text.join(" "),
                embedding: vec![], // will be set at later step
            });
            chunk_text.clear();
            continue;
        }

        chunk_text.push(word.to_string());
    }

    // if theres words left over after the loop ends, add
    if chunk_text.len() > 0 {
        chunks.push(Chunk {
            content: chunk_text.join(" "),
            embedding: vec![], // will be set at later step
        });
        chunk_text.clear();
    }

    chunks
}

pub fn embed_chunks(
    chunks: &mut Vec<Chunk>,
    embedding_model: &mut TextEmbedding,
) -> Result<(), Box<dyn Error>> {
    // split this embedding process into batches
    // will be much faster and prevent massive memory usage
    // some files could potentially have hundreds of thousands of chunks
    // feeding all into the embed method all at once will expload memory usage

    for batch in chunks.chunks_mut(256) {
        let words: Vec<String> = batch.iter().map(|x| x.content.clone()).collect();

        let embeddings: Vec<Vec<f32>> = embedding_model.embed(words, None)?;

        // set the embedding now for each chunk
        for (i, b) in batch.iter_mut().enumerate() {
            b.embedding = embeddings[i].clone();
        }
    }

    Ok(())
}
