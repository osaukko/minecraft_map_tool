use clap::{Parser, Subcommand};
use std::process::ExitCode;

pub mod info_tool;
pub mod list_tool;

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
    Info(info_tool::InfoArgs),

    /// Show information from multiple maps in list form
    List(list_tool::ListArgs),
}

impl Commands {
    fn run(&self) -> ExitCode {
        match self {
            Commands::Info(args) => info_tool::show_info(args),
            Commands::List(args) => list_tool::list_maps(args),
        }
    }
}

fn main() -> ExitCode {
    Cli::parse().command.run()
}
