use clap::Args;
use comfy_table::{presets, Cell, CellAlignment, ContentArrangement, Table, TableComponent};
use crossterm::queue;
use crossterm::style::{Attribute, Print, SetAttribute};
use minecraft_map_tool::MapItem;
use std::{
    io::{stdout, Write},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Show info on this map_#.dat file
    file: PathBuf,
}

pub fn show_info(args: &InfoArgs) -> ExitCode {
    let map_item = match MapItem::read_from(&args.file) {
        Ok(map_item) => map_item,
        Err(err) => {
            eprintln!("Could not read map item: {}", err);
            return ExitCode::FAILURE;
        }
    };

    // Making frames
    let mut frames = Vec::new();
    frames.push(TextFrame {
        title: map_item.file.file_name().unwrap().to_str().unwrap(),
        content: make_basic_info_table(&map_item),
    });
    frames.push(TextFrame {
        title: "Tracking",
        content: make_tracking_table(&map_item),
    });
    frames.push(TextFrame {
        title: "Coordinates (X, Z)",
        content: make_coordinate_table(&map_item),
    });
    if !map_item.data.banners.is_empty() {
        frames.push(TextFrame {
            title: "Banners",
            content: make_banners_table(&map_item),
        });
    }
    if !map_item.data.frames.is_empty() {
        frames.push(TextFrame {
            title: "Frames",
            content: make_frames_table(&map_item),
        });
    }

    // Finding maximum width and set it to all tables
    let mut width = 20; // Minimum width
    for frame in &frames {
        width = std::cmp::max(width, frame.calculate_width())
    }

    // Printing frames
    frames[0].print(width, '╭', '╮');
    for frame in &mut frames[1..] {
        frame.print(width, '├', '┤');
    }
    TextFrame::print_bottom(width, '╰', '╯');

    ExitCode::SUCCESS
}

struct TextFrame<'a> {
    title: &'a str,
    content: Table,
}

impl TextFrame<'_> {
    fn calculate_width(&self) -> u16 {
        let mut width = 0;
        for column_width in self.content.column_max_content_widths() {
            width += column_width + 3; // At least 3 characters between columns
        }
        width - 3 // Removing extra we added in the loop
    }

    fn print(&mut self, width: u16, left: char, right: char) {
        let fill_width = width as usize - self.title.chars().count() - 3;
        let empty_row_width = width as usize + 2;
        queue!(
            stdout(),
            SetAttribute(Attribute::Bold),
            Print(format!(
                "{}──┤ {} ├{:─>fill_width$}\n",
                left, self.title, right
            )),
            Print(format!("│{:empty_row_width$}│\n", ' ')),
        )
        .unwrap();
        self.content.set_width(width);
        self.content
            .set_content_arrangement(ContentArrangement::DynamicFullWidth);
        for line in self.content.lines() {
            queue!(
                stdout(),
                Print("│ "),
                SetAttribute(Attribute::Reset),
                Print(line),
                SetAttribute(Attribute::Bold),
                Print(" │\n"),
            )
            .unwrap();
        }
        queue!(
            stdout(),
            Print(format!("│{:empty_row_width$}│\n", ' ')),
            SetAttribute(Attribute::Reset),
        )
        .unwrap();
    }

    fn print_bottom(width: u16, left: char, right: char) {
        let fill_width = width as usize + 2;
        queue!(
            stdout(),
            SetAttribute(Attribute::Bold),
            Print(format!("{}─{:─>fill_width$}\n", left, right)),
            SetAttribute(Attribute::Reset)
        )
        .unwrap();
        stdout().flush().unwrap();
    }
}

/// Helper function to turn NBT byte to Yes or No string
fn yes_or_no(byte: i8) -> String {
    match byte {
        0 => "No",
        _ => "Yes",
    }
    .to_string()
}

fn make_basic_info_table(map_item: &MapItem) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.add_row(vec![
        "Scale".to_string(),
        map_item.data.scale.to_string(),
        map_item.data.scale_description(),
    ]);
    table.add_row(vec![
        "Version".to_string(),
        map_item.data_version.to_string(),
        map_item.version_description(),
    ]);
    table.add_row(vec![
        "Dimension".to_string(),
        map_item.data.pretty_dimension(),
    ]);
    table.add_row(vec!["Locked".to_string(), yes_or_no(map_item.data.locked)]);
    table
}

fn make_tracking_table(map_item: &MapItem) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.add_row(vec![
        "Tracking position".to_string(),
        yes_or_no(map_item.data.tracking_position),
    ]);
    table.add_row(vec![
        "Unlimited tracking".to_string(),
        yes_or_no(map_item.data.unlimited_tracking),
    ]);
    table
}

fn make_coordinate_table(map_item: &MapItem) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.add_row(vec![
        "Upper (CellAlignment::Left)".to_string(),
        map_item.data.left().to_string(),
        map_item.data.top().to_string(),
    ]);
    table.add_row(vec![
        "Lower (CellAlignment::Left)".to_string(),
        map_item.data.left().to_string(),
        map_item.data.bottom().to_string(),
    ]);
    table.add_row(vec![
        "Upper (CellAlignment::Right)".to_string(),
        map_item.data.right().to_string(),
        map_item.data.top().to_string(),
    ]);
    table.add_row(vec![
        "Lower (CellAlignment::Right)".to_string(),
        map_item.data.right().to_string(),
        map_item.data.bottom().to_string(),
    ]);
    table.add_row(vec![
        "Center".to_string(),
        map_item.data.x_center.to_string(),
        map_item.data.z_center.to_string(),
    ]);
    table
}

fn make_banners_table(map_item: &MapItem) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.set_style(TableComponent::HeaderLines, '╌');
    table.set_style(TableComponent::VerticalLines, ' ');
    table.set_header(vec![
        Cell::new("Name").set_alignment(CellAlignment::Left),
        Cell::new("Color").set_alignment(CellAlignment::Left),
        Cell::new("X").set_alignment(CellAlignment::Right),
        Cell::new("Y").set_alignment(CellAlignment::Right),
        Cell::new("Z").set_alignment(CellAlignment::Right),
    ]);
    for banner in &map_item.data.banners {
        table.add_row(vec![
            Cell::new(banner.extract_name()).set_alignment(CellAlignment::Left),
            Cell::new(banner.color.to_string()).set_alignment(CellAlignment::Left),
            Cell::new(banner.pos.x).set_alignment(CellAlignment::Right),
            Cell::new(banner.pos.y).set_alignment(CellAlignment::Right),
            Cell::new(banner.pos.z).set_alignment(CellAlignment::Right),
        ]);
    }
    table
}

fn make_frames_table(map_item: &MapItem) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table.set_style(TableComponent::HeaderLines, '╌');
    table.set_style(TableComponent::VerticalLines, ' ');
    table.set_header(vec![
        Cell::new("Entity ID").set_alignment(CellAlignment::Left),
        Cell::new("Angle").set_alignment(CellAlignment::Left),
        Cell::new("X").set_alignment(CellAlignment::Right),
        Cell::new("Y").set_alignment(CellAlignment::Right),
        Cell::new("Z").set_alignment(CellAlignment::Right),
    ]);
    for frame in &map_item.data.frames {
        table.add_row(vec![
            Cell::new(frame.entity_id).set_alignment(CellAlignment::Left),
            Cell::new(frame.rotation).set_alignment(CellAlignment::Left),
            Cell::new(frame.pos.x).set_alignment(CellAlignment::Right),
            Cell::new(frame.pos.y).set_alignment(CellAlignment::Right),
            Cell::new(frame.pos.z).set_alignment(CellAlignment::Right),
        ]);
    }
    table
}
