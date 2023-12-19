use clap::{arg, Args};
use fastnbt::ByteArray;
use minecraft_map_tool::versions::MINECRAFT_VERSIONS;
use minecraft_map_tool::{MapData, MapItem};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct TestMapArgs {
    /// Output file name
    #[arg(value_name = "FILE", default_value = "tests/map_0.dat")]
    output_file: PathBuf,

    /// Set data version to [default: latest known version]
    #[arg(short, long, value_name = "VERSION")]
    data_version: Option<i32>,
}

pub fn run(args: &TestMapArgs) -> ExitCode {
    let data_version = match args.data_version {
        None => {
            let mut data_version = 0;
            for key in MINECRAFT_VERSIONS.keys() {
                data_version = std::cmp::max(data_version, *key);
            }
            data_version
        }
        Some(version) => version,
    };

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
        file: args.output_file.clone(),
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
    match test_map.write().map_err(|err| err.to_string()) {
        Ok(_) => {
            println!("Test map written to: {:?}", args.output_file);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Could not write test map: {err}");
            ExitCode::FAILURE
        }
    }
}
