# ROM blob explorer

This is a simple tool for visualizing and exploring binary blobs.

## Building and Installation

`romexp` is written in the Rust language. If you haven't used rust before, you can find simple installation instructions [here](https://www.rust-lang.org/en-US/install.html).

Clone this repository and change directory to the repo's root. Then:
```
$ cargo build --release
$ cargo install
```

Cargo will download and build all the necessary dependencies for you.

## Usage

You can invoke romexp on a binary file by passing it as an argument to romexp2:
```
$ romexp2 [PATH OF FILE]
```

### A quick guide to the interface

You can use the scroll wheel to zoom into the bit view. Dragging the middle mouse button will
pan the view. The offset in the file, in hex, should appear in the lower right corner. Left 
dragging selects a region.

You can annotate the blob with various annotation engines that will highlight appropriate regions
of the code. Mouse over a highlight to see more information.

You can change the byte stride with the left and right keys. Backtick swaps endianness relative to the current byte stride.

Currently implemented annotations:
* S - identify C strings

