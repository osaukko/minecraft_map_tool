use crate::error::{Error, Result};
use crate::palette::Palette;
use crate::versions::MINECRAFT_VERSIONS;
use clap::ValueEnum;
use fastnbt::ByteArray;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use heck::ToTitleCase;
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::VecDeque,
    fs::File,
    path::{Path, PathBuf},
};

pub mod error;
pub mod palette;
pub mod versions;

/// Banner color options
#[derive(Debug, Deserialize, Serialize)]
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
        write!(f, "{self:?}")
    }
}

/// For deserializing banner name from JSON
#[derive(Debug, Deserialize, Serialize)]
struct BannerName {
    text: String,
}

/// A banner marker
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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
            Some(pos) => self.dimension[pos + 1..].replace('_', " ").to_title_case(),
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
#[derive(Debug, Deserialize, Serialize)]
pub struct MapItem {
    /// Path to map file
    ///
    /// **Note:** This is not part of the Minecraft map item and therefore is not serialized.
    #[serde(skip)]
    pub file: PathBuf,

    /// The map data
    pub data: MapData,

    /// The version the map was created
    #[serde(rename = "DataVersion")]
    pub data_version: i32,
}

impl MapItem {
    pub fn make_image(&self, palette: &Palette) -> Result<RgbaImage> {
        let mut image = RgbaImage::new(128, 128);
        let mut color = self.data.colors.iter();
        for y in 0..128 {
            for x in 0..128 {
                let c = *color
                    .next()
                    .ok_or_else(|| Error::map_item_error("Color buffer incomplete"))?
                    as u8;
                let pixel = palette.get(c as usize).unwrap_or(&Rgba([0, 0, 0, 0]));
                image.put_pixel(x, y, *pixel);
            }
        }
        Ok(image)
    }

    /// Pretty dimension from file path
    ///
    /// This function tries to identify the dimension from the file path.
    /// Can be useful for same rare cases.  
    ///
    /// | Path contains   | Name                           |
    /// | --------------- | ------------------------------ |
    /// | _nether         | The Nether                     |
    /// | _the_end        | The End                        |
    /// | (none of above) | `self.data.pretty_dimension()` |
    pub fn pretty_dimension_from_path(&self) -> String {
        let path = self.file.to_string_lossy();
        if path.contains("_nether") {
            String::from("The Nether")
        } else if path.contains("_the_end") {
            String::from("The End")
        } else {
            self.data.pretty_dimension()
        }
    }

    /// Read map item from the given *file* path
    pub fn read_from(file: &Path) -> Result<MapItem> {
        let file_reader = File::open(file)?;
        let decoder = GzDecoder::new(&file_reader);
        let mut map_item: MapItem = fastnbt::from_reader(decoder)?;
        map_item.file = PathBuf::from(file);
        Ok(map_item)
    }

    /// Write map item to custom location
    pub fn write_to(&self, file: &Path) -> Result<()> {
        let file_writer = File::create(file)?;
        let encoder = GzEncoder::new(file_writer, Compression::default());
        fastnbt::to_writer(encoder, self)?;
        Ok(())
    }

    /// Write map item using its [file](MapItem::file) location
    pub fn write(&self) -> Result<()> {
        self.write_to(&self.file)
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
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pos {
    /// The x-position
    pub x: i32,

    /// The y-position
    pub y: i32,

    /// The z-position
    pub z: i32,
}

#[derive(Debug)]
pub struct ReadMap {
    map_files: VecDeque<PathBuf>,
}

impl ReadMap {
    /// Attempts to find a common base path for all map items
    pub fn common_base_path(&self) -> Option<PathBuf> {
        if self.map_files.is_empty() {
            return None;
        }
        let mut iter = self.map_files.iter();
        let mut base = iter.next().unwrap().clone();
        for path in iter {
            let mut new_base = PathBuf::new();
            let a_components = base.components();
            let b_components = path.components();
            let zipped = a_components.zip(b_components);
            for (a, b) in zipped {
                if a == b {
                    new_base.push(a)
                }
            }
            base = new_base;
        }
        Some(base)
    }

    pub fn file_count(&self) -> usize {
        self.map_files.len()
    }

    pub fn from_paths(map_files: VecDeque<PathBuf>) -> ReadMap {
        ReadMap { map_files }
    }

    pub fn is_empty(&self) -> bool {
        self.map_files.is_empty()
    }
}

impl Iterator for ReadMap {
    type Item = Result<MapItem>;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_files
            .pop_front()
            .map(|path| MapItem::read_from(&path))
    }
}

pub fn read_maps(path: &Path, sort: &Option<SortingOrder>, recursive: bool) -> Result<ReadMap> {
    let mut directory_stack = VecDeque::new();
    let mut map_files = VecDeque::new();
    directory_stack.push_back(PathBuf::from(path));
    while !directory_stack.is_empty() {
        let dir = directory_stack.pop_front().unwrap();
        let read_dir = match dir.read_dir() {
            Ok(read_dir) => read_dir,
            Err(err) => {
                eprintln!("Warning: Could not read: {dir:?}, {err}");
                continue;
            }
        };
        for dir_entry in read_dir.flatten() {
            let path = dir_entry.path();
            if path.is_symlink() {
                // We do not follow symlinks for now, could cause forever loop
                continue;
            } else if path.is_file()
                && path.extension().unwrap_or_default() == "dat"
                && path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .starts_with("map_")
            {
                map_files.push_back(dir_entry.path());
            } else if path.is_dir() && recursive {
                directory_stack.push_back(dir_entry.path());
            }
        }
    }
    if let Some(sort) = sort {
        map_files.make_contiguous().sort_by(|a, b| sort.cmp(a, b));
    }
    Ok(ReadMap { map_files })
}

/// Sorting order for map files
#[derive(Clone, Debug, ValueEnum)]
pub enum SortingOrder {
    /// Files are organized by name and numbers in the natural order
    Name,

    /// Files are organized from oldest to newest
    Time,
}

impl SortingOrder {
    /// This method returns an Ordering between *a* and *b* path based on *self* value.
    pub fn cmp(&self, a: &Path, b: &Path) -> Ordering {
        match self {
            SortingOrder::Name => {
                let a_str = a.as_os_str().to_str().expect("invalid path");
                let b_str = b.as_os_str().to_str().expect("invalid path");
                natord::compare(a_str, b_str)
            }
            SortingOrder::Time => {
                let a_modified = &a
                    .metadata()
                    .expect("could not read metadata")
                    .modified()
                    .expect("could not get modification time");
                let b_modified = &b
                    .metadata()
                    .expect("could not read metadata")
                    .modified()
                    .expect("could not get modification time");
                a_modified.cmp(b_modified)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::palette::{generate_palette, BASE_COLORS_2699};
    use crate::MapItem;
    use image::{GenericImageView, Pixel};
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_make_image() {
        let map_item = MapItem::read_from(&project_file(&Path::new("tests/map_0.dat"))).unwrap();
        let map_image = map_item
            .make_image(&generate_palette(&BASE_COLORS_2699))
            .unwrap();
        let reference_image = image::open(&project_file(&Path::new("tests/map_0.png"))).unwrap();
        assert_eq!(map_image.dimensions(), reference_image.dimensions());

        // Comparing each pixel and collecting wrong colors to map
        let mut wrong_colors = BTreeMap::new();
        for y in 0..128 {
            for x in 0..128 {
                let map_pixel = *map_image.get_pixel(x, y);
                let reference_pixel = reference_image.get_pixel(x, y);
                if map_pixel != reference_pixel {
                    // Find the color value that is wrong
                    let color = *map_item.data.colors.get((y * 128 + x) as usize).unwrap();
                    wrong_colors
                        .entry(color as u8)
                        .or_insert((map_pixel, reference_pixel));
                }
            }
        }

        // Panic if wrong colors is not empty
        if !wrong_colors.is_empty() {
            // Formatting errors for easy to read format
            let mut wrong_colors_message = format!(
                "{:<8}#{:<12}#{:<12}\n",
                "Color", "What it is", "What it should be"
            );
            for (color, (left, right)) in wrong_colors {
                wrong_colors_message.push_str(&format!(
                    "{:<8}#{:<12}#{:<12}\n",
                    color,
                    hex::encode(left.channels()),
                    hex::encode(right.channels())
                ))
            }
            panic!("{}", wrong_colors_message);
        }
    }

    fn project_file(path: &Path) -> PathBuf {
        let mut relative_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        relative_path.push(path);
        relative_path
    }
}
