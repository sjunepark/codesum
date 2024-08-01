use codesum::Cli;

use clap::Parser;

fn main() {
    let cli = Cli::parse();
    println!("Path: {}", cli.path);
}
