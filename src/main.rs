use clap::{Parser, Subcommand};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};
use image::RgbaImage;
use minecraft_map_tool::error::{Error, Result};
use minecraft_map_tool::MinecraftMapper;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(version, long_about = None)]
#[command(about = "This program tells information about map files and creates images from them")]
struct Cli {
    /// Which action should the tool take?
    #[command(subcommand)]
    command: Commands,

    /// Directory where map data files are
    #[arg(short, long, default_value = "data", value_name = "PATH")]
    input_dir: PathBuf,

    /// Output directory where image(s) are written
    #[arg(short, long, default_value = "images", value_name = "PATH")]
    output_dir: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List maps and their information
    List,

    /// Create one image from multiple maps
    Image {
        /// Use maps with scale (Map zoom level)
        scale: i8,

        /// Left coordinate (Smaller X)
        left: i32,

        /// Top coordinate (Smaller Z)
        top: i32,

        /// Right coordinate (Larger X)
        right: i32,

        /// bottom coordinate (Larger Z)
        bottom: i32,

        /// Filename for the image (Written to output directory)
        filename: String,
    },

    /// Create an image from each map
    Images,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    println!("Command    : {:?}", cli.command);
    println!("Input dir  : {}", cli.input_dir.display());
    println!("Output dir : {}", cli.output_dir.display());
    println!();

    let result = match cli.command {
        Commands::List => list_maps(&cli),
        Commands::Image { .. } => make_image(&cli),
        Commands::Images => make_images(&cli),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}

fn list_maps(cli: &Cli) -> Result<()> {
    let minecraft_mapper = MinecraftMapper::new();
    let maps = minecraft_mapper.read_maps(&cli.input_dir)?;
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "File",
            "Zoom",
            "Dimension",
            "Locked",
            "Center",
            "Left",
            "Top",
            "Right",
            "Bottom",
        ]);
    for map in maps {
        table.add_row(vec![
            Cell::new(map.file.file_name().unwrap().to_str().unwrap()),
            Cell::new(map.scale),
            Cell::new(map.dimension.replace("minecraft:", "")),
            Cell::new(map.locked),
            Cell::new(format!("{}, {}", map.center.0, map.center.1)),
            Cell::new(map.left()),
            Cell::new(map.top()),
            Cell::new(map.right()),
            Cell::new(map.bottom()),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn make_images(cli: &Cli) -> Result<()> {
    let minecraft_mapper = MinecraftMapper::new();
    let maps = minecraft_mapper
        .read_maps(&cli.input_dir)
        .map_err(|err| format!("Could not read maps: {}", err))?;
    ensure_output_dir(cli)?;
    for map in maps {
        if let Ok(image) = minecraft_mapper.make_image(&map) {
            let output_filename = format!(
                "{}/{}_{}_{}_{}_{}.png",
                cli.output_dir.display(),
                map.dimension.replace("minecraft:", ""),
                map.scale,
                map.center.0,
                map.center.1,
                map.file.file_stem().unwrap_or_default().to_str().unwrap()
            );
            println!("{} -> {}", map.file.display(), output_filename);
            image.save(output_filename)?;
        }
    }
    Ok(())
}

fn make_image(cli: &Cli) -> Result<()> {
    if let Commands::Image {
        scale,
        left,
        top,
        right,
        bottom,
        filename,
    } = &cli.command
    {
        if *scale != 0 {
            return Err(Error::from(
                "Currently only scale 0 is supported".to_string(),
            ));
        }

        if left >= right || top >= bottom {
            return Err(Error::from("Invalid coordinates".to_string()));
        }

        let image_width = (right - left + 1) as u32;
        let image_height = (bottom - top + 1) as u32;

        println!("Output image size: {}×{}", image_width, image_height);
        let mut image = RgbaImage::new(image_width, image_height);

        let minecraft_mapper = MinecraftMapper::new();
        let maps = minecraft_mapper
            .read_maps(&cli.input_dir)
            .map_err(|err| format!("Could not read maps: {}", err))?;
        for map in maps {
            if map.scale != *scale {
                continue;
            }
            if map.left() <= *right
                && map.top() <= *bottom
                && map.right() >= *left
                && map.bottom() >= *top
            {
                println!(
                    "Adding {:?} [l: {}, t: {}, r: {}, b: {}]",
                    map.file.file_name().unwrap(),
                    map.left(),
                    map.top(),
                    map.right(),
                    map.bottom()
                );
                let map_image = minecraft_mapper.make_image(&map)?;
                paint_image(&map_image, &mut image, map.left() - left, map.top() - top);
            }
        }

        let mut output_filename = cli.output_dir.clone();
        output_filename.push(filename);
        println!("Saving to: {:?}", output_filename);
        ensure_output_dir(cli)?;
        image.save(output_filename)?;
    }
    Ok(())
}

fn ensure_output_dir(cli: &Cli) -> Result<()> {
    fs::create_dir_all(&cli.output_dir)?;
    Ok(())
}

fn paint_image(source: &RgbaImage, target: &mut RgbaImage, x: i32, y: i32) {
    for in_y in 0..source.height() {
        for in_x in 0..source.width() {
            let out_x = in_x as i32 + x;
            let out_y = in_y as i32 + y;
            if out_x < 0
                || out_y < 0
                || out_x as u32 >= target.width()
                || out_y as u32 >= target.height()
            {
                continue; // Outside of the target image
            }
            let pixel = source.get_pixel(in_x, in_y);
            if pixel[3] == 0 {
                continue; // Transparent
            }
            target.put_pixel(out_x as u32, out_y as u32, *pixel);
        }
    }
}
