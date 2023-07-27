use clap::Parser;
use std::path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // Execute command
    pub execute: String,

    // file / directory output path
    #[arg(short, default_value = ".")]
    pub path: path::PathBuf,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}
