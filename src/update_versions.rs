use clap::{arg, Args};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct UpdateVersionsArgs {
    /// Download the versions from
    #[arg(
        short,
        long,
        value_name = "URL",
        default_value = "https://minecraft.fandom.com/wiki/Data_version"
    )]
    source_url: String,

    /// Output file name
    #[arg(short, long, value_name = "FILE", default_value = "src/versions.rs")]
    output_file: PathBuf,
}

pub fn run(args: &UpdateVersionsArgs) -> ExitCode {
    println!("Loading: {}", args.source_url);
    let body = match load(&args.source_url) {
        Ok(body) => body,
        Err(err) => {
            eprintln!("Loading error: {err}");
            return ExitCode::FAILURE;
        }
    };

    // The XML reader runs to errors if we try to parse the whole page.
    // Therefore, we try to find the table and pass it to the XML reader.
    let versions_table = match find_version_table(&body) {
        Ok(table) => table,
        Err(err) => {
            eprintln!("Could not find version table: {err}");
            return ExitCode::FAILURE;
        }
    };

    // Parsing versions from the XML
    let mut buf = Vec::new();
    let mut versions_tree = BTreeMap::new();
    let mut table_row = TableRow::new();
    let mut reader = Reader::from_str(versions_table);
    loop {
        match reader.read_event_into(&mut buf) {
            // Stop at error
            Err(err) => {
                eprintln!("XML error: {err}");
                return ExitCode::FAILURE;
            }

            // Parse rows when start of 'tr' is found
            Ok(Event::Start(event)) => {
                if event.name().as_ref() == b"tr" {
                    match table_row.read(&mut reader, &mut buf) {
                        Ok(_) => {
                            if let Ok(version_info) = table_row.parse_line() {
                                versions_tree
                                    .entry(version_info.data_version)
                                    .or_insert(version_info.client_version);
                            }
                        }
                        Err(err) => {
                            eprintln!("Error while parsing table row: {err}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
            }

            // Exits the loop when reaching "end of file"
            Ok(Event::Eof) => break,

            // Continue loop for rest of the events
            _ => (),
        }
    }

    let mut versions_code = r#"use phf::{phf_map, Map};

/// Mapping data versions to known client versions
///
/// The table was made from the content available at
/// [https://minecraft.fandom.com/wiki/Data_version](https://minecraft.fandom.com/wiki/Data_version#List_of_data_versions)
pub const MINECRAFT_VERSIONS: Map<i32, &'static str> = phf_map! {
"#.to_string();
    for (data_version, client_version) in versions_tree {
        versions_code.push_str(&format!("    {data_version}i32 => \"{client_version}\",\n"));
    }
    versions_code.push_str("};\n");

    match fs::write(&args.output_file, versions_code) {
        Ok(_) => {
            println!("Source code written to: {:?}", args.output_file);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("Error while writing source code: {err}");
            ExitCode::FAILURE
        }
    }
}

fn err_to_string<E>(err: E) -> String
where
    E: Display,
{
    err.to_string()
}

fn load(url: &str) -> Result<String, String> {
    let response = reqwest::blocking::get(url).map_err(err_to_string)?;
    response.text().map_err(err_to_string)
}

fn find_version_table(body: &str) -> Result<&str, &str> {
    let heading_pos = match body.rfind("List of data versions") {
        None => return Err("Could not find the 'List of data versions' heading"),
        Some(pos) => pos,
    };
    let start_pos = match body[heading_pos..].find("<table") {
        None => return Err("Could not find table after the 'List of data versions' heading"),
        Some(pos) => heading_pos + pos,
    };
    let end_pos = match body[start_pos..].find("</table>") {
        None => return Err("Could not find end of the versions table"),
        Some(pos) => start_pos + pos + 8,
    };

    Ok(&body[start_pos..end_pos])
}

struct VersionInfo {
    client_version: String,
    data_version: i32,
}

#[derive(Debug)]
struct TableRow {
    cells: Vec<TableCell>,
}

impl TableRow {
    fn new() -> TableRow {
        TableRow {
            cells: vec![TableCell::new(), TableCell::new(), TableCell::new()],
        }
    }

    fn read(&mut self, reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<(), String> {
        for cell in self.cells.iter_mut() {
            cell.read(reader, buf)?;
        }
        Ok(())
    }

    fn parse_line(&self) -> Result<VersionInfo, &'static str> {
        if self.cells[0].text.is_empty() {
            return Err("Client version is empty");
        }
        let client_version = self.cells[0].text.clone();
        let data_version = self.cells[2]
            .text
            .parse::<i32>()
            .map_err(|_| "Could not parse data version")?;
        Ok(VersionInfo {
            client_version,
            data_version,
        })
    }
}

#[derive(Debug)]
struct TableCell {
    text: String,
    rowspan: u32,
}

impl TableCell {
    fn new() -> TableCell {
        TableCell {
            text: "".to_string(),
            rowspan: 0,
        }
    }

    fn read(&mut self, reader: &mut Reader<&[u8]>, buf: &mut Vec<u8>) -> Result<(), String> {
        if self.rowspan > 0 {
            self.rowspan -= 1;
            return Ok(());
        }
        let mut td_found = false;
        loop {
            match reader.read_event_into(buf) {
                // Stop at error
                Err(err) => return Err(format!("XML error: {err}")),

                // We should not run to the "End of File"
                Ok(Event::Eof) => return Err("Unexpected end of file".to_string()),

                Ok(Event::Start(event)) => {
                    if event.name().as_ref() == b"td" {
                        if td_found {
                            return Err("Unexpected second td tag".to_string());
                        } else {
                            td_found = true;
                            for attribute_result in event.attributes() {
                                let attribute = attribute_result.map_err(err_to_string)?;
                                if attribute.key.as_ref() == b"rowspan" {
                                    self.rowspan = attribute
                                        .decode_and_unescape_value(reader)
                                        .map_err(err_to_string)?
                                        .parse::<u32>()
                                        .map_err(err_to_string)?
                                        - 1;
                                }
                            }
                        }
                    }
                }

                Ok(Event::Text(event)) => {
                    if td_found {
                        self.text = event.unescape().map_err(err_to_string)?.to_string();
                    }
                }

                Ok(Event::End(event)) => match event.name().as_ref() {
                    b"td" => {
                        return if td_found {
                            Ok(())
                        } else {
                            Err("Unexpected end td tag found".to_string())
                        }
                    }
                    b"th" => {
                        self.text.clear();
                        return Ok(());
                    }
                    b"tr" => return Err("Unexpected tr tag".to_string()),
                    _ => (),
                },

                _ => (),
            }
        }
    }
}
