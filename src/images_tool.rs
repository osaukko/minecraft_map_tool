use clap::{arg, Args};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct ImagesArgs {
    /// The directory from which map files are searched for
    path: PathBuf,

    /// Output directory. Default is the current directory.
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    /// Search map files recursively in subdirectories
    #[arg(short, long)]
    recursive: bool,
}

pub fn run(args: &ImagesArgs) -> ExitCode {
    eprintln!("TODO! {args:#?}");
    ExitCode::FAILURE
}
