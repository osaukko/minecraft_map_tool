# Minecraft Map Tool

![Map image](docs/title.png "A map made with the example command")

This repository has a program that can tell you information about the maps in the Minecraft game and draw pictures from them. I am developing this project for enjoyment and learning more about Rust and GitHub.

With luck, this program will also be helpful for others. Please note that the program handles the game's map items, the same ones we fill in the game. The program does not draw maps from the chunks.

## Build

The program is written in rust. If you do not already have rust, we recommend you install it using `rustup`. The official rust [book](https://doc.rust-lang.org/book/) contains [instructions](https://doc.rust-lang.org/book/ch01-01-installation.html) on how to install rust on Linux, macOS, and Windows.

To build use the following command:

```bash
cargo build --release
```

We can now find tool binaries under to directory `target/release/`

## How to Use

Start by filling maps in the game.

![A Minecraft player holding a large number of maps](docs/maps.png "Maps, lots of maps")

### Show Info

```
Show information on map_#.dat file

Usage: minecraft_map_tool info <FILE>

Arguments:
  <FILE>  Show info on this map_#.dat file

Options:
  -h, --help  Print help
```

```bash
minecraft_map_tool info tests/map_0.dat
```

### Show Info for Multiple files

```
Show information from multiple maps in list form

Usage: minecraft_map_tool list [OPTIONS] <PATH>

Arguments:
  <PATH>
          The directory from which map files are searched for

Options:
  -r, --recursive
          Search map files recursively in subdirectories

  -s, --sort <SORT>
          Sorting order for files
          
          [default: name]

          Possible values:
          - name: Files are organized by name and numbers in the natural order
          - time: Files are organized from oldest to newest

  -h, --help
          Print help (see a summary with '-h')
```

```bash
minecraft_map_tool list -r -s name ~/Games/Minecraft/
```

### Make Map Image

```
Create an image from a single map file

Usage: minecraft_map_tool image [OPTIONS] <MAP_FILE>

Arguments:
  <MAP_FILE>  Create image of this map_#.dat file

Options:
  -o, --output-file <OUTPUT_FILE>  Write the map image to the file. Standard file formats are supported
  -s, --show-in-terminal           Show map in terminal using iTerm, Kitty, or Sixel graphics protocol
  -h, --help                       Print help
```

```bash
 minecraft_map_tool image tests/map_0.dat -o test.png
```

