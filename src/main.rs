use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod image_tool;
mod images_tool;
mod info_tool;
mod list_tool;
mod stitching_tool;

#[cfg(feature = "dev_tools")]
mod test_map;

#[cfg(feature = "dev_tools")]
mod update_versions;

#[derive(Debug, Parser)]
#[command(version, about)]
#[command(long_about = "\
This program allows you to print information from Minecraft map files
or create images from them.

Add one of the commands listed in the Commands section below and the
command's arguments after the minecraft_map_tool program. 

To get help with the commands, use the help command followed by the
command name you want help with. Or you can also enter -h or --help
after the command.

Examples:
  minecraft_map_tool help info
  minecraft_map_tool list -h
  minecraft_map_tool image --help")]
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

    /// Create an image from a single map file
    Image(image_tool::ImageArgs),

    /// Create images from multiple map files
    Images(images_tool::ImagesArgs),

    /// Drawing multiple maps into a single image
    Stitch(stitching_tool::StitchingArgs),

    /// Create test map item with all colors
    #[cfg(feature = "dev_tools")]
    TestMap(test_map::TestMapArgs),

    /// Attempts to download the versions list from
    /// the Minecraft Wiki and update the versions source file.
    #[cfg(feature = "dev_tools")]
    UpdateVersions(update_versions::UpdateVersionsArgs),
}

impl Commands {
    fn run(&self) -> ExitCode {
        match self {
            // Default tools
            Commands::Info(args) => info_tool::run(args),
            Commands::Image(args) => image_tool::run(args),
            Commands::Images(args) => images_tool::run(args),
            Commands::List(args) => list_tool::run(args),
            Commands::Stitch(args) => stitching_tool::run(args),

            // Development tools
            #[cfg(feature = "dev_tools")]
            Commands::TestMap(args) => test_map::run(args),

            #[cfg(feature = "dev_tools")]
            Commands::UpdateVersions(args) => update_versions::run(args),
        }
    }
}

fn main() -> ExitCode {
    Cli::parse().command.run()
}
