use std::{error::Error};
use pdfsink_rs::PdfDocument;

struct Chunk {
    raw_text: String
}
struct ChunkResult {
    chunks: Vec<Chunk>
}

pub fn read_pdf(path: &str) -> Result<String, Box<dyn Error>> {
    let file = PdfDocument::open(path)?;
    let text = file.extract_text();
    Ok(text)
}

pub fn build_chunk_prompt(arg: &str) -> ChunkResult {
    // this function will build out a vector of 250 word "chunks"
    // from a massive string argument

    let words = arg.split("\n");
    let str = String::new();
}
