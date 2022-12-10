use crate::error::Result;
use crate::versions::MINECRAFT_VERSIONS;
use fastnbt::ByteArray;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub mod error;
pub mod palette;
pub mod versions;

/// Banner color options
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BannerColor {
    Black,
    Blue,
    Brown,
    Cyan,
    Gray,
    Green,
    LightBlue,
    LightGray,
    Lime,
    Magenta,
    Orange,
    Pink,
    Purple,
    Red,
    White,
    Yellow,
}

impl std::fmt::Display for BannerColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// For deserializing banner name
#[derive(Debug, Deserialize)]
struct BannerName {
    text: String,
}

/// A banner marker
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Banner {
    /// The color of the banner.
    pub color: BannerColor,

    /// The custom name of the banner, in JSON text. May not exist.
    pub name: Option<String>,

    /// The block position of the banner in the world.
    pub pos: Pos,
}

impl Banner {
    /// Returns the name only
    ///
    /// Names are stored into JSON, and this function tries to extract the name out of JSON.
    /// If banner does not have name, then `[nameless]` is returned.
    ///
    /// If name parsing from JSON fails, then error message is returned as name
    pub fn extract_name(&self) -> String {
        match &self.name {
            None => "[nameless]".to_string(),
            Some(json) => match serde_json::from_str::<BannerName>(json) {
                Ok(banner_name) => banner_name.text,
                Err(error) => error.to_string(),
            },
        }
    }
}

/// The map data
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapData {
    /// How zoomed in the map is (it is in 2<sup>scale</sup> wide blocks square per pixel,
    /// even for 0, where the map is 1:1). Minimum 0 and maximum 4.
    pub scale: i8,

    /// For <1.16 (byte): 0 = The Overworld, -1 = The Nether, 1 = The End,
    /// any other value = a static image with no player pin.
    /// In >=1.16 this is the resource location of a dimension instead.
    pub dimension: String,

    /// 1 indicates that a positional arrow should be shown when the map is near its
    /// center coords. 0 indicates that the position arrow should never be shown.
    pub tracking_position: i8,

    /// 1 allows the player position indicator to show as a smaller dot on the map's edge when the
    /// player is farther than 320 * (scale+1) blocks from the map's center. 0 makes the dot instead
    /// disappear when the player is farther than this distance.
    pub unlimited_tracking: i8,

    /// 1 if the map has been locked in a cartography table.
    pub locked: i8,

    /// Center of map according to real world by X.
    pub x_center: i32,

    /// Center of map according to real world by Z.
    pub z_center: i32,

    /// List of banner markers added to this map. May be empty.
    pub banners: Vec<Banner>,

    /// List map markers added to this map. May be empty.
    pub frames: Vec<Marker>,

    /// Width * Height array of color values (16384 entries for a default 128×128 map).
    pub colors: ByteArray,
}

impl MapData {
    /// Scale description in format of 1:1, 1:2, etc.
    pub fn scale_description(&self) -> String {
        format!("1:{}", 2i32.pow(self.scale as u32))
    }

    /// Pretty dimension
    ///
    /// Returns `Overworld` instead of `minecraft:overworld`
    pub fn pretty_dimension(&self) -> String {
        match self.dimension.find(':') {
            None => self.dimension.clone(),
            Some(pos) => {
                self.dimension[pos + 1..pos + 2].to_uppercase() + &self.dimension[pos + 2..]
            }
        }
    }

    /// X coordinate for pixels on the left edge of the map
    pub fn left(&self) -> i32 {
        self.x_center - 64 * 2i32.pow(self.scale as u32)
    }

    /// Z coordinate for pixels on the top edge of the map
    pub fn top(&self) -> i32 {
        self.z_center - 64 * 2i32.pow(self.scale as u32)
    }

    /// X coordinate for pixels on the right edge of the map
    pub fn right(&self) -> i32 {
        self.x_center + 64 * 2i32.pow(self.scale as u32) - 1
    }

    /// Z coordinate for pixels on the bottom edge of the map
    pub fn bottom(&self) -> i32 {
        self.z_center + 64 * 2i32.pow(self.scale as u32) - 1
    }
}

/// Custom debug implementation to avoid printing all 16384 color values
impl std::fmt::Debug for MapData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct ColorsDebug<'a>(&'a ByteArray);
        impl std::fmt::Debug for ColorsDebug<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "[{} bytes]", self.0.len())
            }
        }
        f.debug_struct("Map Data")
            .field("scale", &self.scale)
            .field("dimension", &self.dimension)
            .field("tracking_position", &self.tracking_position)
            .field("unlimited_tracking", &self.unlimited_tracking)
            .field("locked", &self.locked)
            .field("x_center", &self.x_center)
            .field("z_center", &self.z_center)
            .field("banners", &self.banners)
            .field("frames", &self.frames)
            .field("colors", &ColorsDebug(&self.colors))
            .finish()
    }
}

/// Content of the map_<#>.dat files
#[derive(Debug, Deserialize)]
pub struct MapItem {
    /// Path to map file (not part of the item itself)
    #[serde(skip)]
    pub file: PathBuf,

    /// The map data
    pub data: MapData,

    /// The version the map was created
    #[serde(rename = "DataVersion")]
    pub data_version: i32,
}

impl MapItem {
    /// Read map item from the given *file* path
    pub fn read_from(file: &Path) -> Result<MapItem> {
        // Read map file
        let compressed_data = fs::read(file)?;

        // Uncompress data
        let mut gz = GzDecoder::new(&compressed_data[..]);
        let mut uncompressed_data = Vec::new();
        gz.read_to_end(&mut uncompressed_data)?;

        // The version the map was created
        let mut map_item: MapItem = fastnbt::from_bytes(uncompressed_data.as_slice())?;
        map_item.file = PathBuf::from(file);
        Ok(map_item)
    }

    /// Version description
    ///
    /// Returns version name from the [MINECRAFT_VERSIONS] table
    /// or "Unknown" if matching version is not found.
    pub fn version_description(&self) -> String {
        MINECRAFT_VERSIONS
            .get(&self.data_version)
            .unwrap_or(&"Unknown")
            .to_string()
    }
}

/// A marker
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Marker {
    /// Arbitrary unique value for the marker.
    pub entity_id: i32,

    /// The rotation of the marker, ranging from 0 to 360.
    pub rotation: i32,

    /// The rotation of the marker, ranging from 0 to 360.
    pub pos: Pos,
}

/// Position coordinate in the Minecraft world
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pos {
    /// The x-position
    pub x: i32,

    /// The y-position
    pub y: i32,

    /// The z-position
    pub z: i32,
}
