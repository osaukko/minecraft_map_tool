use clap::{arg, Args};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL_CONDENSED;
use comfy_table::{Cell, ContentArrangement, Table};
use minecraft_map_tool::{read_maps, SortingOrder};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// The directory from which map files are searched for
    path: PathBuf,

    /// Search map files recursively in subdirectories
    #[arg(short, long)]
    recursive: bool,

    /// Sorting order for files
    #[arg(short, long, default_value = "name")]
    sort: SortingOrder,
}

pub fn list_maps(args: &ListArgs) -> ExitCode {
    let maps = match read_maps(&args.path, &args.sort, args.recursive) {
        Ok(maps) => maps,
        Err(err) => {
            eprintln!("Could not get maps: {}", err);
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
        .load_preset(UTF8_FULL_CONDENSED)
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
    for map in maps.flatten() {
        let file = match map.file.strip_prefix(&common_base_path) {
            Ok(file) => file,
            Err(_) => map.file.as_path(),
        };
        table.add_row(vec![
            Cell::new(file.display()),
            Cell::new(map.data.scale),
            Cell::new(map.data.pretty_dimension()),
            Cell::new(map.data.locked),
            Cell::new(format!("{}, {}", map.data.x_center, map.data.z_center)),
            Cell::new(map.data.left()),
            Cell::new(map.data.top()),
            Cell::new(map.data.right()),
            Cell::new(map.data.bottom()),
        ]);
    }
    println!("{table}");
    ExitCode::SUCCESS
}
