use clap::{arg, Args};
use comfy_table::{Cell, ContentArrangement, Table};
use minecraft_map_tool::{read_maps, SortingOrder};
use std::path::PathBuf;
use std::process::ExitCode;

#[cfg(not(target_os = "windows"))]
const PRESET: &str = "││──╞═╪╡┆    ┬┴╭╮╰╯";

// In Windows, rounded corners will work if the user has changed the command prompt to use
// a UTF-8 compatible font. However, by default, this is not the case; therefore, we use
// rectangular borders instead.
#[cfg(target_os = "windows")]
const PRESET: &str = "││──├─┼┤│    ┬┴┌┐└┘";

#[derive(Args, Debug)]
pub struct ListArgs {
    /// The directory from which map files are searched for
    path: PathBuf,

    /// Search map files recursively in subdirectories
    #[arg(short, long)]
    recursive: bool,

    /// Sorting order for files
    #[arg(short, long, default_value = "name")]
    sort: Option<SortingOrder>,

    /// Try to detect world dimensions from the file path instead of map item data.
    #[arg(short, long)]
    dimension_from_path: bool,
}

pub fn run(args: &ListArgs) -> ExitCode {
    let maps = match read_maps(&args.path, &args.sort, args.recursive) {
        Ok(maps) => maps,
        Err(err) => {
            eprintln!("Could not get maps: {err}");
            return ExitCode::FAILURE;
        }
    };
    if maps.is_empty() {
        println!("Nothing to list");
        return ExitCode::FAILURE;
    }
    let common_base_path = maps.common_base_path().unwrap_or_default();
    let mut table = Table::new();
    table
        .load_preset(PRESET)
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
            "Banners",
            "Frames",
        ]);
    for map in maps.flatten() {
        let file = match map.file.strip_prefix(&common_base_path) {
            Ok(file) => file,
            Err(_) => map.file.as_path(),
        };
        table.add_row(vec![
            Cell::new(file.display()),
            Cell::new(map.data.scale),
            Cell::new(if args.dimension_from_path {
                map.pretty_dimension_from_path()
            } else {
                map.data.pretty_dimension()
            }),
            Cell::new(map.data.locked),
            Cell::new(format!("{}, {}", map.data.x_center, map.data.z_center)),
            Cell::new(map.data.left()),
            Cell::new(map.data.top()),
            Cell::new(map.data.right()),
            Cell::new(map.data.bottom()),
            Cell::new(map.data.banners.len()),
            Cell::new(map.data.frames.len()),
        ]);
    }
    println!("{table}");
    ExitCode::SUCCESS
}
