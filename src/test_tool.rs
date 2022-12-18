use clap::{arg, Args};
use fastnbt::ByteArray;
use minecraft_map_tool::versions::MINECRAFT_VERSIONS;
use minecraft_map_tool::{MapData, MapItem};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct TestArgs {
    /// Create test map item with all colors
    #[arg(short, long, value_name = "FILE")]
    make_test_map: Option<PathBuf>,
}

pub fn test_tool(args: &TestArgs) -> ExitCode {
    if let Some(file) = &args.make_test_map {
        if let Err(message) = make_test_map(file) {
            eprintln!("Could not write test map: {}", message);
            return ExitCode::FAILURE;
        }
        println!("Test map written to: {:?}", file);
    }
    ExitCode::SUCCESS
}

fn make_test_map(file: &Path) -> Result<(), String> {
    let mut data_version = 0;
    for key in MINECRAFT_VERSIONS.keys() {
        data_version = std::cmp::max(data_version, *key);
    }

    // Generating map with all colors, where each color have 8x8 pixels
    let mut colors: Vec<i8> = Vec::with_capacity(128 * 128);
    let mut color = 0u8;
    for _ in 0..16 {
        let mut line = Vec::with_capacity(128);
        for _ in 0..16 {
            for _ in 0..8 {
                line.push(color as i8);
            }
            color = color.wrapping_add(1);
        }
        for _ in 0..8 {
            colors.extend(&line);
        }
    }

    let test_map = MapItem {
        file: PathBuf::from(file),
        data: MapData {
            scale: 0,
            dimension: "minecraft:overworld".to_string(),
            tracking_position: 1,
            unlimited_tracking: 0,
            locked: 1,
            x_center: 0,
            z_center: 0,
            banners: vec![],
            frames: vec![],
            colors: ByteArray::new(colors),
        },
        data_version,
    };
    test_map.write().map_err(|err| err.to_string())?;
    Ok(())
}
