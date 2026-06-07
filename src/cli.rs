use std::{error::Error, path::PathBuf};

use clap::Parser;
use fastembed::{
    EmbeddingModel::{self, BGESmallENV15},
    ModelInfo, TextEmbedding,
};

use crate::utils::VALID_FILE_EXTENSIONS;

#[derive(Parser, Debug)]
pub struct Args {
    pub path: Option<String>,

    #[arg(long)]
    pub exts: Option<String>, // comma seperated string of all the file extensions to parse

    #[arg(long)]
    pub model: Option<String>, // underlying embedding model to use (must be supported by fastembed crate)
}

pub struct CliConfig {
    pub path_to_parse: PathBuf,
    pub exts_to_parse: Vec<String>,
    pub model_to_use: ModelInfo<EmbeddingModel>,
}

pub fn get_cli_config(cli_args: &Args) -> Result<CliConfig, Box<dyn Error>> {
    let user_defined_path = &cli_args.path;
    let user_defined_exts = &cli_args.exts;

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
            if !VALID_FILE_EXTENSIONS.contains(e) {
                return Err(format!("{} is not a supported file extension", e).into());
            }
            exts_to_parse.push(e.to_string());
        }
    }

    // set the model
    // must be a valid supported model from the fastembed crate
    let model_to_use: ModelInfo<EmbeddingModel>;
    if cli_args.model.is_none() {
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
            .find(|x| x.model_code == cli_args.model.to_owned().unwrap());

        if models.is_none() {
            return Err(format!(
                "{} is not a supported embedding model",
                cli_args.model.to_owned().unwrap()
            )
            .into());
        }

        model_to_use = models.unwrap();
    }

    Ok(CliConfig {
        path_to_parse,
        exts_to_parse,
        model_to_use,
    })
}
