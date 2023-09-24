use clap::Args;
use image::DynamicImage;
use minecraft_map_tool::palette::{generate_palette, BASE_COLORS_2699};
use minecraft_map_tool::MapItem;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct ImageArgs {
    /// Create image of this map_#.dat file
    map_file: PathBuf,

    /// Write the map image to the file. Standard file formats are supported.
    #[arg(short, long)]
    output_file: Option<PathBuf>,

    /// Show map in terminal
    #[arg(short, long, group = "term")]
    show_in_terminal: bool,
}

pub fn run(args: &ImageArgs) -> ExitCode {
    let map_item = match MapItem::read_from(&args.map_file) {
        Ok(map_item) => map_item,
        Err(err) => {
            eprintln!("Could not read map item: {err}");
            return ExitCode::FAILURE;
        }
    };

    let image = match map_item.make_image(&generate_palette(&BASE_COLORS_2699)) {
        Ok(image) => image,
        Err(err) => {
            eprintln!("Could not create image: {err}");
            return ExitCode::FAILURE;
        }
    };

    if args.show_in_terminal {
        let config = viuer::Config {
            absolute_offset: false,
            transparent: true,
            truecolor: true,
            ..Default::default()
        };
        let dynamic_image = DynamicImage::from(image.clone());
        if let Err(err) = viuer::print(&dynamic_image, &config) {
            eprintln!("Could not show image: {err}");
            return ExitCode::FAILURE;
        }
    }

    if let Some(output_file) = &args.output_file {
        match image.save(output_file) {
            Ok(_) => println!("Image written to: {output_file:?}"),
            Err(err) => {
                eprintln!("Could not write image: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
