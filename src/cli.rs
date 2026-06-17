use std::{error::Error, path::PathBuf};

use clap::{Parser, Subcommand};
use fastembed::{
    EmbeddingModel::{self, BGESmallENV15},
    ModelInfo, TextEmbedding,
};

use crate::{
    constants::{DOCUMENT_EXTENSIONS, TEXT_EXTENSIONS},
    utils::DatabaseType,
};

pub struct CliChunkConfig {
    pub path_to_parse: PathBuf,
    pub exts_to_parse: Vec<String>,
    pub model_to_use: ModelInfo<EmbeddingModel>,
    pub chunk_size: i32,
    pub database_type: DatabaseType,
}

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Chunk {
        #[arg(long)]
        database_url: String,

        #[arg(long)]
        path: Option<String>,

        #[arg(long)]
        exts: Option<String>,

        #[arg(long)]
        model: Option<String>,

        #[arg(long)]
        size: Option<i32>,

        #[arg(long)]
        require_ssl: bool,
    },

    List {},
}

impl Commands {
    pub fn get_cli_chunk_config(
        path: Option<String>,
        exts: Option<String>,
        model: Option<String>,
        size: Option<i32>,
        database_url: &str,
        valid_file_extensions: &Vec<&str>
    ) -> Result<CliChunkConfig, Box<dyn Error>> {
        let user_defined_path = path;
        let user_defined_exts = exts;

        // set the custom path to parse files from
        // if not set... just parse all files in cwd
        let cwd: PathBuf = std::env::current_dir()?;
        let path_to_parse: std::path::PathBuf;
        if let Some(x) = user_defined_path {
            path_to_parse = cwd.join(x);
        } else {
            path_to_parse = cwd;
        }

        // allow the user to set the file extensions they want to parse
        // all exts passed here have to be "valid"
        let mut exts_to_parse: Vec<String> = vec![];
        if let Some(x) = user_defined_exts {
            let exts: Vec<&str> = x.split(",").collect();
            // make sure each extension is valid
            for e in &exts {
                if !valid_file_extensions.contains(e) {
                    return Err(format!("{} is not a supported file extension", e).into());
                }
                exts_to_parse.push(e.to_string());
            }
        }

        // set the model
        // must be a valid supported model from the fastembed crate
        let model_to_use: ModelInfo<EmbeddingModel>;
        if model.is_none() {
            // if user doesnt pass a model, default to BGESmallENV15
            let models = TextEmbedding::list_supported_models()
                .into_iter()
                .find(|x| x.model == BGESmallENV15);

            if models.is_none() {
                return Err(format!("error while setting default embedding model").into());
            }

            model_to_use = models.unwrap();
        } else {
            let models = TextEmbedding::list_supported_models()
                .into_iter()
                .find(|x| x.model_code == model.clone().unwrap());

            if models.is_none() {
                return Err(format!(
                    "{} is not a supported embedding model",
                    model.clone().unwrap()
                )
                .into());
            }

            model_to_use = models.unwrap();
        }

        // set hte chunk size
        let chunk_size: i32;
        if size.is_none() {
            chunk_size = 250 // defualt
        } else {
            chunk_size = size.unwrap();
        }

        // determine the database type
        let database_type: DatabaseType;
        if database_url.starts_with("postgres") {
            database_type = DatabaseType::Postgres
        } else if database_url.starts_with("mysql") {
            database_type = DatabaseType::Mysql
        } else {
            return Err(format!("invalid database connection string").into());
        }

        Ok(CliChunkConfig {
            path_to_parse,
            exts_to_parse,
            model_to_use,
            chunk_size,
            database_type,
        })
    }
}
