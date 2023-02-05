use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod image_tool;
mod info_tool;
mod list_tool;

#[cfg(feature = "test_tool")]
mod test_tool;

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

    /// Create an image from a single map file
    Image(image_tool::ImageArgs),

    /// Show information from multiple maps in list form
    List(list_tool::ListArgs),

    /// Tools to help program testing
    #[cfg(feature = "test_tool")]
    Test(test_tool::TestArgs),
}

impl Commands {
    fn run(&self) -> ExitCode {
        match self {
            Commands::Info(args) => info_tool::show_info(args),
            Commands::Image(args) => image_tool::create_image(args),
            Commands::List(args) => list_tool::list_maps(args),
            #[cfg(feature = "test_tool")]
            Commands::Test(args) => test_tool::test_tool(args),
        }
    }
}

fn main() -> ExitCode {
    Cli::parse().command.run()
}
