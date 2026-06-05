use docx_lite::{ExtractOptions, parse_document_from_path};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use pdfsink_rs::PdfDocument;
use std::{
    error::Error,
    ffi::OsString,
    fs::{self, DirEntry, File},
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

pub const VALID_FILE_EXTENSIONS: [&str; 4] = ["txt", "pdf", "docx", "pptx"];

#[derive(Debug)]
pub struct FileDetail {
    pub extension: String,
    pub absolute_path: String,
}

#[derive(Debug)]
pub struct Chunk {
    pub content: String,     // raw human readable text
    pub embedding: Vec<f32>, // the vectors that represents this chunk of text
}

#[derive(PartialEq)]
pub enum EmbeddingModelUsed {
    BGESmallENV15,
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
                absolute_path: path_str,
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
        absolute_path: path_str,
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

pub fn chunk_text_file(file: &FileDetail) -> Result<Vec<Chunk>, Box<dyn Error>> {
    let f: File = File::open(&file.absolute_path)?;
    let mut buf_reader: BufReader<File> = BufReader::new(f);

    let mut str = String::new();
    buf_reader.read_to_string(&mut str)?;

    let mut chunks: Vec<Chunk> = vec![];
    let mut chunk_text: Vec<String> = vec![];

    extract_250_word_chunks(&mut chunks, &mut chunk_text, str);

    // now that have all the chunks, need to embed each one
    // loop through the reutnred value and update to the embedding field on the struct
    embed_chunks(&mut chunks)?;

    return Ok(chunks);
}

pub fn chunk_pdf_file(file: &FileDetail) -> Result<Vec<Chunk>, Box<dyn Error>> {
    let pdf = PdfDocument::open(file.absolute_path.clone())?;
    if pdf.is_empty() {
        // no pages on pdf, just return success with no chunks
        return Ok(vec![]);
    }
    let str = pdf.extract_text();

    let mut chunks: Vec<Chunk> = vec![];
    let mut chunk_text: Vec<String> = vec![];

    extract_250_word_chunks(&mut chunks, &mut chunk_text, str);

    // now that have all the chunks, need to embed each one
    // loop through the reutnred value and update to the embedding field on the struct
    embed_chunks(&mut chunks)?;

    Ok(chunks)
}

pub fn chunk_docx_file(file: &FileDetail) -> Result<Vec<Chunk>, Box<dyn Error>> {
    let doc: docx_lite::Document = parse_document_from_path(file.absolute_path.clone())?;
    let str: String = doc.extract_text();

    let mut chunks: Vec<Chunk> = vec![];
    let mut chunk_text: Vec<String> = vec![];

    extract_250_word_chunks(&mut chunks, &mut chunk_text, str);

    // now that have all the chunks, need to embed each one
    // loop through the reutnred value a-> Result<Vec<Chunk>, Box<dyn Error>>nd update to the embedding field on the struct
    embed_chunks(&mut chunks)?;

    return Ok(chunks);
}

pub fn chunk_pptx_file(file: &FileDetail) -> Result<Vec<Chunk>, Box<dyn Error>> {
    let ppt = rustypptx::parse_pptx(Path::new(&file.absolute_path))?;

    let str: String = ppt.to_markdown();

    let mut chunks: Vec<Chunk> = vec![];
    let mut chunk_text: Vec<String> = vec![];

    extract_250_word_chunks(&mut chunks, &mut chunk_text, str);

    // now that have all the chunks, need to embed each one
    // loop through the reutnred value a-> Result<Vec<Chunk>, Box<dyn Error>>nd update to the embedding field on the struct
    embed_chunks(&mut chunks)?;

    return Ok(chunks);
}

fn extract_250_word_chunks(chunks: &mut Vec<Chunk>, chunk_text: &mut Vec<String>, str: String) {
    // helper function to extract entire text from files parsed into 250 chunks
    // before sending into embedding model

    for word in str.split_whitespace() {
        if chunk_text.len() >= 250 {
            // once hit 250 words, build the next chunk
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
}

fn embed_chunks(chunks: &mut Vec<Chunk>) -> Result<(), Box<dyn Error>> {
    let mut model: TextEmbedding =
        TextEmbedding::try_new(InitOptions::new(EmbeddingModel::BGESmallENV15))?;

    let mut words: Vec<String> = vec![];
    for c in chunks.iter_mut() {
        words.push(c.content.clone())
    }

    let embeddings = model.embed(words, None)?;
    let num_of_chunks = chunks.len();
    let num_of_embeddings = embeddings.len();

    if num_of_embeddings != num_of_chunks {
        return Err("embedding count did not match chunk count".into());
    }

    // set the embedding now for each chunk
    for (i, c) in chunks.iter_mut().enumerate() {
        c.embedding = embeddings[i].clone();
    }

    Ok(())
}
