use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
/// Aggregates all the code within a certain path.
///
/// You can filter out files by their extension, and exclude any path that contains
pub struct Cli {
    pub path: String,
}
