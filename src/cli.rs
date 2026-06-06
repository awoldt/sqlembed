use std::{error::Error, path::PathBuf};

use clap::Parser;

use crate::utils::VALID_FILE_EXTENSIONS;

#[derive(Parser, Debug)]
pub struct Args {
    pub path: Option<String>,

    #[arg(long)]
    pub exts: Option<String>, // comma seperated string of all the file extensions to parse
}

pub struct CliConfig {
    pub path_to_parse: PathBuf,
    pub exts_to_parse: Vec<String>,
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

    Ok(CliConfig {
        path_to_parse,
        exts_to_parse,
    })
}
