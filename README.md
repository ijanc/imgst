# imgst

`imgst` is a command-line tool for processing and metadata removal from
image files (currently JPEG).

It recursively scans a directory, removes metadata such as EXIF, and writes the
cleaned files into a separate output directory while preserving the folder
structure.

## Why

When publishing images on websites or sharing them publicly, embedded metadata
(EXIF) may leak sensitive information:

- GPS coordinates
- Device information (camera model, serial)
- Date and time the photo was taken
- Thumbnails containing past edits
- Optional personal identifiers

Images uploaded from mobile devices frequently include **location metadata**,
which can expose private home or workplace addresses.

Additionally, metadata increases file size, affecting storage and page load
performance.

### Safer and lighter images

By stripping metadata:

- No privacy-leaking metadata remains
- Storage and bandwidth usage shrink
- Website performance improves

## Features

- Recursive directory scanning with ignore rules (`.gitignore`, `.ignore`)
- JPEG EXIF metadata removal (`web-image-meta`)
- Preserves the directory hierarchy
- Parallel processing for performance
- Dry-run mode shows what would be processed without modifying files
- Optional statistics (`--stats`) including total space savings
- Logging with adjustable verbosity (`-v`, `RUST_LOG`)

## Requirements

- Rust toolchain (Rust 1.89 or newer recommended)

To install Rust:

```sh
curl https://sh.rustup.rs -sSf | sh
```

Verify installation:

```sh
rustc --version
cargo --version
```

## Building

Clone the repository and build the binary:

```sh
git clone ssh://anon@ijanc.org/imgst
cd imgst
cargo build --release
```

The binary will be located at:

```
target/release/imgst
```

You can add it to your PATH or move it to `/usr/local/bin`.

or

```
cargo install --path .
```

## Command Overview

`imgst` currently supports one primary operation:

### Metadata Cleaning

Recursively removes metadata from image files and writes the result to a
separate output folder:

```sh
imgst --input ./photos --output ./public/photos
```

Preserves folder structure:

```
photos/
 └── albums/
     └── party/
         └── img_001.jpg

public/photos/
 └── albums/
     └── party/
         └── img_001.jpg  <-- cleaned
```

### Dry-run mode

Shows what would be processed but does not write anything:

```sh
imgst -i ./photos -o ./out --dry-run -v
```

### Statistics mode

Display space savings after completion:

```sh
imgst -i ./photos -o ./out --stats
```

Example output:

```
Stats:
  Original total : 9.83 GB
  Clean total    : 8.97 GB
  Saved          : 860 MB (8.7%)
```

## Logging and verbosity

`imgst` uses standard Rust logging (`env_logger`).

Default level: **INFO**

Use `-v` to enable DEBUG:

```sh
imgst -v -i ./photos -o ./out
```

Or configure directly:

```sh
RUST_LOG=debug imgst ...
```

## License

Licensed under the ISC license\
(see [`LICENSE`](LICENSE) or https://opensource.org/licenses/ISC)
