mod cli;

use crate::cli::Args;

fn main() {
    let args = Args::new();

    println!("Execute command: {:?}", args.execute);
}
