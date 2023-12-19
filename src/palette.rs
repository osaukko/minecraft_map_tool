use image::Rgba;
use phf::{phf_map, Map};

const MULTIPLIERS: [u16; 4] = [180, 220, 255, 135];

/// Palette can be generated from base colors
pub type BaseColors = Map<u8, [u8; 4]>;

/// Palette has color for all possible values for map pixel
pub type Palette = [Rgba<u8>; 256];

/// Base color as they were in [21w10a](https://minecraft.fandom.com/wiki/Java_Edition_21w10a)
/// version (Data version 2699)
///
/// Source: [https://minecraft.fandom.com/wiki/Map_item_format](https://minecraft.fandom.com/wiki/Map_item_format)
pub const BASE_COLORS_2699: BaseColors = phf_map! {
        1u8 => [127, 178, 56, 255],
        2u8 => [247, 233, 163, 255],
        3u8 => [199, 199, 199, 255],
        4u8 => [255, 0, 0, 255],
        5u8 => [160, 160, 255, 255],
        6u8 => [167, 167, 167, 255],
        7u8 => [0, 124, 0, 255],
        8u8 => [255, 255, 255, 255],
        9u8 => [164, 168, 184, 255],
        10u8 => [151, 109, 77, 255],
        11u8 => [112, 112, 112, 255],
        12u8 => [64, 64, 255, 255],
        13u8 => [143, 119, 72, 255],
        14u8 => [255, 252, 245, 255],
        15u8 => [216, 127, 51, 255],
        16u8 => [178, 76, 216, 255],
        17u8 => [102, 153, 216, 255],
        18u8 => [229, 229, 51, 255],
        19u8 => [127, 204, 25, 255],
        20u8 => [242, 127, 165, 255],
        21u8 => [76, 76, 76, 255],
        22u8 => [153, 153, 153, 255],
        23u8 => [76, 127, 153, 255],
        24u8 => [127, 63, 178, 255],
        25u8 => [51, 76, 178, 255],
        26u8 => [102, 76, 51, 255],
        27u8 => [102, 127, 51, 255],
        28u8 => [153, 51, 51, 255],
        29u8 => [25, 25, 25, 255],
        30u8 => [250, 238, 77, 255],
        31u8 => [92, 219, 213, 255],
        32u8 => [74, 128, 255, 255],
        33u8 => [0, 217, 58, 255],
        34u8 => [129, 86, 49, 255],
        35u8 => [112, 2, 0, 255],
        36u8 => [209, 177, 161, 255],
        37u8 => [159, 82, 36, 255],
        38u8 => [149, 87, 108, 255],
        39u8 => [112, 108, 138, 255],
        40u8 => [186, 133, 36, 255],
        41u8 => [103, 117, 53, 255],
        42u8 => [160, 77, 78, 255],
        43u8 => [57, 41, 35, 255],
        44u8 => [135, 107, 98, 255],
        45u8 => [87, 92, 92, 255],
        46u8 => [122, 73, 88, 255],
        47u8 => [76, 62, 92, 255],
        48u8 => [76, 50, 35, 255],
        49u8 => [76, 82, 42, 255],
        50u8 => [142, 60, 46, 255],
        51u8 => [37, 22, 16, 255],
        52u8 => [189, 48, 49, 255],
        53u8 => [148, 63, 97, 255],
        54u8 => [92, 25, 29, 255],
        55u8 => [22, 126, 134, 255],
        56u8 => [58, 142, 140, 255],
        57u8 => [86, 44, 62, 255],
        58u8 => [20, 180, 133, 255],
        59u8 => [100, 100, 100, 255],
        60u8 => [216, 175, 147, 255],
        61u8 => [127, 167, 150, 255],
};

pub fn generate_palette(base_colors: &BaseColors) -> Palette {
    let mut palette: Palette = [Rgba([0u8; 4]); 256];
    for i in 0..64 {
        // Color components are mapped to u16 so that we have enough bits for math operations,
        // final color components are u8
        let base_color = match base_colors.get(&i) {
            None => [0u16; 4], // Using transparent for missing colors
            Some([r, g, b, a]) => [*r as u16, *g as u16, *b as u16, *a as u16],
        };
        for (j, multiplier) in MULTIPLIERS.iter().enumerate() {
            for (k, channel) in base_color.iter().enumerate() {
                palette[i as usize * 4 + j][k] = if k == 3 {
                    // Alpha channel is passed as it is
                    *channel as u8
                } else {
                    // For other channels, a multiplier is applied
                    ((channel * multiplier) / 255) as u8
                };
            }
        }
    }
    palette
}
