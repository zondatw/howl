use clap::Parser;
use std::path;

use nix::sys::signal::Signal;

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

    // signal, ref: https://man7.org/linux/man-pages/man7/signal.7.html
    // signal doc: https://docs.rs/nix/latest/nix/sys/signal/enum.Signal.html#variants
    #[arg(short = 's', default_value_t = Signal::SIGINT)]
    pub signal: Signal,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}
