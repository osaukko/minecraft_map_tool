use clap::{arg, Args};
use minecraft_map_tool::palette::{generate_palette, BASE_COLORS_2699};
use minecraft_map_tool::read_maps;
use std::fs;
use std::path::{Path, PathBuf};
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
    // Collect map information
    let maps = match read_maps(&args.path, &None, args.recursive) {
        Ok(maps) => maps,
        Err(err) => {
            eprintln!("Could not get maps: {err}");
            return ExitCode::FAILURE;
        }
    };
    if maps.is_empty() {
        println!("Could not find any maps!");
        return ExitCode::FAILURE;
    }

    // Prepare palette
    let palette = generate_palette(&BASE_COLORS_2699);

    // Process maps
    for map in maps.flatten() {
        let mut output_dir = args.output_dir.clone().unwrap_or_default();
        if args.recursive {
            output_dir.push(PathBuf::from(map.pretty_dimension()));
        }
        let output_file =
            Path::join(&output_dir, &map.file.file_stem().unwrap()).with_extension("png");
        if let Err(error) = fs::create_dir_all(output_dir) {
            eprintln!("Could not create output directory: {error}");
            return ExitCode::FAILURE;
        }
        let image = match map.make_image(&palette) {
            Ok(image) => image,
            Err(err) => {
                eprintln!("Could not create image: {err}");
                return ExitCode::FAILURE;
            }
        };
        match image.save(&output_file) {
            Ok(_) => println!("Image written to: {output_file:?}"),
            Err(err) => {
                eprintln!("Could not write image: {output_file:?}\n{err}");
                return ExitCode::FAILURE;
            }
        };
    }

    // Done
    ExitCode::SUCCESS
}
