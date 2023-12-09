use clap::{arg, Args};
use minecraft_map_tool::SortingOrder;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct StitchingArgs {
    /// Only draw maps with matching dimensions name
    #[arg(short, long, default_value = "Overworld")]
    dimension: Option<String>,

    /// Search map files recursively in subdirectories
    #[arg(long)]
    recursive: bool,

    /// Image drawing order
    #[arg(short, long, default_value = "time")]
    sort: Option<SortingOrder>,

    /// Draw only maps with this zoom level
    #[arg(short, long, default_value_t = 0)]
    zoom: i8,

    /// Left coordinate (Smaller X)
    #[arg(short, long)]
    left: Option<i32>,

    /// Top coordinate (Smaller Z)
    #[arg(short, long)]
    top: Option<i32>,

    /// Right coordinate (Larger X)
    #[arg(short, long)]
    right: Option<i32>,

    /// bottom coordinate (Larger Z)
    #[arg(short, long)]
    bottom: Option<i32>,

    /// The directory from which map files are searched for
    path: PathBuf,

    /// Filename for the output image
    filename: String,
}

pub fn run(args: &StitchingArgs) -> ExitCode {
    eprintln!("TODO! {args:#?}");
    ExitCode::FAILURE
}
