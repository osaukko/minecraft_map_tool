use anyhow::{anyhow, Result};
use clap::{arg, Args};
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use minecraft_map_tool::palette::{generate_palette, BASE_COLORS_2699};
use minecraft_map_tool::{read_maps, ReadMap, SortingOrder};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

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

struct ImageProject {
    maps: ReadMap,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

fn filter_and_area(
    maps: ReadMap,
    scale: i8,
    dimension: &Option<String>,
) -> anyhow::Result<ImageProject> {
    // Making dimension to lowercase for case-insensitive comparison
    let dimension = dimension.clone().map(|s| s.to_lowercase());

    // Container for filtered map paths
    let mut filtered_map_files: VecDeque<PathBuf> = VecDeque::new();

    // Variables for finding the map area
    let mut left = i32::MAX;
    let mut top = i32::MAX;
    let mut right = i32::MIN;
    let mut bottom = i32::MIN;

    for map_item in maps.flatten() {
        // Filtering with scale
        if map_item.data.scale != scale {
            continue;
        }

        // Filtering with dimension
        if let Some(dimension) = &dimension {
            if &map_item.data.pretty_dimension().to_lowercase() != dimension {
                continue;
            }
        }

        // Update map area
        left = left.min(map_item.data.left());
        top = top.min(map_item.data.top());
        right = right.max(map_item.data.right());
        bottom = bottom.max(map_item.data.bottom());

        // Keep this map item in new list
        filtered_map_files.push_back(map_item.file);
    }

    if filtered_map_files.is_empty() {
        return Err(anyhow!("No map files after filtering"));
    }

    let maps = ReadMap::from_paths(filtered_map_files);
    Ok(ImageProject {
        maps,
        left,
        top,
        right,
        bottom,
    })
}

fn prepare(args: &StitchingArgs) -> Result<ImageProject> {
    if args.zoom != 0 {
        return Err(anyhow!("Only zoom step 0 is currently supported"));
    }

    // Get maps
    let maps = read_maps(&args.path, &args.sort, args.recursive)
        .map_err(|err| anyhow!(format!("Could not read maps: {err}")))?;
    if maps.is_empty() {
        return Err(anyhow!("No map files found"));
    }
    println!("Found {} map files.", maps.file_count());

    // Filtering and finding the area
    let ImageProject {
        maps,
        mut left,
        mut top,
        mut right,
        mut bottom,
    } = filter_and_area(maps, args.zoom, &args.dimension)?;
    println!("After filtering we have {} map files.", maps.file_count());
    println!("Map area");
    println!("  Upper Left  : {left} {top}");
    println!("  Lower Right : {right} {bottom}");
    println!("  Size        : {}×{}", right - left + 1, bottom - top + 1);

    // Apply users area limits if given
    if let Some(value) = args.left {
        left = value;
    }
    if let Some(value) = args.top {
        top = value;
    }
    if let Some(value) = args.right {
        right = value;
    }
    if let Some(value) = args.bottom {
        bottom = value;
    }
    println!("Map area for image");
    println!("  Upper Left  : {left} {top}");
    println!("  Lower Right : {right} {bottom}");
    println!("  Size        : {}×{}", right - left + 1, bottom - top + 1);

    Ok(ImageProject {
        maps,
        left,
        top,
        right,
        bottom,
    })
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

fn make_image(project: ImageProject) -> Result<RgbaImage> {
    // Create Image
    let width = (project.right - project.left + 1) as u32;
    let height = (project.bottom - project.top + 1) as u32;
    println!("Making image with size: {width}×{height}");
    let mut image = RgbaImage::new(width, height);

    // Prepare palette
    let palette = generate_palette(&BASE_COLORS_2699);

    // Painting maps
    let progress_bar = ProgressBar::new(project.maps.file_count() as u64);
    progress_bar.set_style(ProgressStyle::with_template(
        "{spinner:.green} {msg} [{bar:40.green}] {pos}/{len} ({eta})",
    )?);
    progress_bar.set_message("Drawing maps");

    for map_item in project.maps.flatten() {
        if map_item.data.left() <= project.right
            && map_item.data.top() <= project.bottom
            && map_item.data.right() >= project.left
            && map_item.data.bottom() >= project.top
        {
            // Map overlaps the target image, paint it
            let map_image = map_item
                .make_image(&palette)
                .map_err(|err| anyhow!("Could not paint image: {err}"))?;
            paint_image(
                &map_image,
                &mut image,
                map_item.data.left() - project.left,
                map_item.data.top() - project.top,
            );
        }
        progress_bar.inc(1);
    }
    progress_bar.finish();

    Ok(image)
}

fn process(args: &StitchingArgs) -> Result<()> {
    let project = prepare(args)?;
    let image = make_image(project)?;
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(ProgressStyle::with_template("{spinner:.green} {msg}")?);
    progress_bar.set_message(format!("Saving image as {:?}", args.filename));
    progress_bar.enable_steady_tick(Duration::from_millis(50));
    image.save(&args.filename)?;
    progress_bar.finish();
    Ok(())
}

pub fn run(args: &StitchingArgs) -> ExitCode {
    // Try to make the image
    if let Err(err) = process(args) {
        eprintln!("{err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
