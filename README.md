# Minecraft Map Tool

![Map image](docs/title.png "A map made with the example command")

This repository has a program that can tell you information about the maps in the Minecraft game and draw pictures from them. I am developing this project for enjoyment and learning more about Rust and GitHub.

With luck, this program will also be helpful for others. Please note that the program handles the game's map items, the same ones we fill in the game. The program does not draw maps from the chunks.

## How to use

Start by filling maps in the game.

![A Minecraft player holding a large number of maps](docs/maps.png "Maps, lots of maps")

Use the application to generate an image from them.

```bash
$ minecraft_map_tool -i /path/to/map/data/ image -- 0 -52 -1087 747 -788 title.png
```

The program tells you how to use it if you ask for help from it.

```bash
$ minecraft_map_tool help                                                                                                          
This program tells information about map files and creates images from them

Usage: minecraft_map_tool [OPTIONS] <COMMAND>

Commands:
  list    List maps and their information
  image   Create one image from multiple maps
  images  Create an image from each map
  help    Print this message or the help of the given subcommand(s)

Options:
  -i, --input-dir <PATH>   Directory where map data files are [default: data]
  -o, --output-dir <PATH>  Output directory where image(s) are written [default: images]
  -h, --help               Print help information
  -V, --version            Print version information
```
