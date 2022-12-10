use crate::info_tool::{show_info, InfoArgs};
use clap::{Parser, Subcommand};
use std::process::ExitCode;

pub mod info_tool;

#[derive(Debug, Parser)]
#[command(version, long_about = None)]
#[command(about = "This program tells information about map files and creates images from them")]
struct Cli {
    /// Which action should the tool take?
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show information on map_#.dat file
    Info(InfoArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info(args) => show_info(args),
    }
}
