use anyhow::Result;
use clap::Args;
use fastnbt::stream::{ErrorKind, Name, Parser, Value};
use flate2::read::GzDecoder;
use ptree::{print_tree, TreeBuilder};
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct DumpNbtArgs {
    /// Path to the NBT file to inspect
    nbt_file: PathBuf,
}

pub fn run(args: &DumpNbtArgs) -> ExitCode {
    match dump_nbt(&args.nbt_file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Could not dump NBT file: {err}");
            ExitCode::FAILURE
        }
    }
}

fn dump_nbt(file: &Path) -> Result<()> {
    let file_reader = File::open(file)?;
    let decoder = GzDecoder::new(&file_reader);
    let mut parser = Parser::new(decoder);

    let filename = filename_to_string(&file)?;
    let mut tree = TreeBuilder::new(filename);

    loop {
        match parser.next() {
            Ok(value) => {
                match value {
                    Value::Compound(name) => { tree.begin_child(format!("Compound: {}", name.unwrap_or_default())); }
                    Value::CompoundEnd => { tree.end_child(); }

                    Value::List(name, tag, count) => { tree.begin_child(format!("List: {} [{tag:?}]×{count}", name.unwrap_or_default())); }
                    Value::ListEnd => { tree.end_child(); }

                    Value::Byte(name, value) => { tree.add_empty_child(format!("Byte: {} = {value}", name.unwrap_or_default())); }
                    Value::Short(name, value) => { tree.add_empty_child(format!("Short: {} = {value}", name.unwrap_or_default())); }
                    Value::Int(name, value) => { tree.add_empty_child(format!("Int: {} = {value}", name.unwrap_or_default())); }
                    Value::Long(name, value) => { tree.add_empty_child(format!("Long: {} = {value}", name.unwrap_or_default())); }
                    Value::Float(name, value) => { tree.add_empty_child(format!("Float: {} = {value}", name.unwrap_or_default())); }
                    Value::Double(name, value) => { tree.add_empty_child(format!("Double: {} = {value}", name.unwrap_or_default())); }
                    Value::String(name, value) => { tree.add_empty_child(format!("String: {} = {value:?}", name.unwrap_or_default())); }

                    Value::ByteArray(name, values) => { tree.add_empty_child(format_array("ByteArray", name, &values)); }
                    Value::IntArray(name, values) => { tree.add_empty_child(format_array("IntArray", name, &values)); }
                    Value::LongArray(name, values) => { tree.add_empty_child(format_array("LongArray", name, &values)); }
                }
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::Eof => {}
                    _ => eprintln!("{err:?}"),
                }
                break;
            }
        }
    }

    print_tree(&tree.build())?;

    Ok(())
}

fn filename_to_string(path: &Path) -> Result<String> {
    let os_str = path.file_name().ok_or_else(|| anyhow::anyhow!("Path has no filename"))?;
    let filename = os_str.to_str().ok_or_else(|| anyhow::anyhow!("Filename is not valid UTF-8"))?;
    Ok(filename.to_string())
}

fn format_array<T: std::fmt::Debug>(type_name: &str, value_name: Name, array: &Vec<T>) -> String {
    if array.len() < 8 {
        format!("{type_name}: {} = {array:?}", value_name.unwrap_or_default())
    } else {
        format!("{type_name}: {} = [{} values]", value_name.unwrap_or_default(), array.len())
    }
}