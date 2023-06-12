use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    // Execute command
    pub execute: String,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}
