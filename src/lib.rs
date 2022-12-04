use crate::error::{Error, Result};
use crate::palette::Palette;
use flate2::read::GzDecoder;
use image::{Rgba, RgbaImage};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use valence_nbt::{from_binary_slice, Value};

pub mod error;
pub mod palette;

/// Some of the data available from map_<#>.dat files
#[derive(Debug)]
pub struct MapItem {
    /// Map file path
    pub file: PathBuf,

    /// How zoomed in the map is
    pub scale: i8,

    /// Map item dimension
    pub dimension: String,

    /// Is the map has been locked in a cartography table?
    pub locked: bool,

    /// Map center (x, z)
    pub center: (i32, i32),

    /// Pixel data
    pub colors: Vec<i8>,
}

impl MapItem {
    pub fn left(&self) -> i32 {
        // TODO: This have been only tested for scale 0. Math for other scale values should be checked.
        self.center.0 - 64 * 2i32.pow(self.scale as u32)
    }

    pub fn top(&self) -> i32 {
        // TODO: This have been only tested for scale 0. Math for other scale values should be checked.
        self.center.1 - 64 * 2i32.pow(self.scale as u32)
    }

    pub fn right(&self) -> i32 {
        // TODO: This have been only tested for scale 0. Math for other scale values should be checked.
        self.center.0 + 64 * 2i32.pow(self.scale as u32)
    }

    pub fn bottom(&self) -> i32 {
        // TODO: This have been only tested for scale 0. Math for other scale values should be checked.
        self.center.1 + 64 * 2i32.pow(self.scale as u32)
    }
}

pub struct MinecraftMapper {
    palette: Palette,
}

impl MinecraftMapper {
    pub fn new() -> MinecraftMapper {
        MinecraftMapper {
            palette: palette::generate(),
        }
    }

    pub fn make_image(&self, map_item: &MapItem) -> Result<RgbaImage> {
        let mut image = RgbaImage::new(128, 128);
        let mut color = map_item.colors.iter();
        for y in 0..128 {
            for x in 0..128 {
                let c = *color
                    .next()
                    .ok_or_else(|| Error::invalid_data("Color buffer incomplete"))?
                    as u8;
                let pixel = self.palette.get(&c).unwrap_or(&Rgba([0, 0, 0, 0]));
                image.put_pixel(x, y, *pixel);
            }
        }
        Ok(image)
    }

    pub fn read_maps(&self, path: &Path) -> Result<Vec<MapItem>> {
        // Make human sorted list of map files
        let mut map_files = Vec::new();
        for entry in (path.read_dir()?).flatten() {
            if entry.path().extension().unwrap_or_default() == "dat"
                && entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .starts_with("map_")
            {
                map_files.push(entry.path());
            }
        }
        map_files
            .sort_by(|a, b| natord::compare(&a.display().to_string(), &b.display().to_string()));

        // Load map items
        let mut maps = Vec::new();
        for map_file in map_files {
            if let Ok(map_item) = self.read_map(&map_file) {
                maps.push(map_item);
            }
        }

        if maps.is_empty() {
            Err(Error::invalid_data("Maps not found"))
        } else {
            Ok(maps)
        }
    }

    pub fn read_map(&self, file: &Path) -> Result<MapItem> {
        // Read map file
        let compressed_data = fs::read(file)?;

        // Uncompress data
        let mut gz = GzDecoder::new(&compressed_data[..]);
        let mut uncompressed_data = Vec::new();
        gz.read_to_end(&mut uncompressed_data)?;

        // Read NBT
        let (nbt, _) = from_binary_slice(&mut uncompressed_data.as_slice())?;
        if let Value::Compound(data) = nbt
            .get("data")
            .ok_or_else(|| Error::invalid_data("data compound not found"))?
        {
            let scale = match data.get("scale") {
                Some(Value::Byte(value)) => value,
                _ => return Err(Error::invalid_data("Could not read scale")),
            };
            let locked = match data.get("locked") {
                Some(Value::Byte(value)) => value,
                _ => return Err(Error::invalid_data("Could not read locked")),
            };
            let x_center = match data.get("xCenter") {
                Some(Value::Int(value)) => value,
                _ => return Err(Error::invalid_data("Could not read xCenter")),
            };
            let z_center = match data.get("zCenter") {
                Some(Value::Int(value)) => value,
                _ => return Err(Error::invalid_data("Could not read zCenter")),
            };
            let dimension = match data.get("dimension") {
                Some(Value::String(value)) => value,
                _ => return Err(Error::invalid_data("Could not read dimension")),
            };
            let colors = match data.get("colors") {
                Some(Value::ByteArray(value)) => value,
                _ => return Err(Error::invalid_data("Could not read colors")),
            };

            return Ok(MapItem {
                file: PathBuf::from(file),
                scale: scale.to_owned(),
                dimension: dimension.to_owned(),
                locked: *locked != 0,
                center: (*x_center, *z_center),
                colors: colors.to_owned(),
            });
        }

        Err(Error::invalid_data(
            "This file did not have known Minecraft map item",
        ))
    }
}

impl Default for MinecraftMapper {
    fn default() -> Self {
        Self::new()
    }
}
