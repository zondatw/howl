use clap::Parser;
use std::path;

use crate::contents::enums;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // Execute command
    pub execute: String,

    // file event
    #[arg(short = 'e', value_enum, default_value_t = enums::FileEvent::Modify)]
    pub file_event: enums::FileEvent,

    // file / directory output path
    #[arg(short, default_value = ".")]
    pub path: path::PathBuf,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}
