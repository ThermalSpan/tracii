extern crate clap;
extern crate glob;
extern crate image; 
extern crate rusttype;
extern crate tempdir;

mod args_and_usage;

use rusttype::{Font, FontCollection, SharedBytes};
use std::fs::File;
use std::io::Read;
use std::process::exit;

fn main() {
	// If we cannot build args, the program will perform a non-zero exit
    let args = args_and_usage::parse_args();

	let mut font_file = File::open(&args.font_path).unwrap();
	let mut byte_buffer = Vec::new();
	
    if let Err(read_error) = font_file.read_to_end(&mut byte_buffer) {
        println!("There was an error reading from {}\n{}", args.font_path.to_string_lossy(), read_error);
        exit(2);
    }

	let mut fonts: Vec<Font> = FontCollection::from_bytes(SharedBytes::ByRef(&byte_buffer))
		.into_fonts()
		.collect();

	if fonts.len() == 0 {
		println!("There were no fonts in {}", args.font_path.to_string_lossy());
		exit(2);
	}

	if fonts.len() > 1 {
		println!("There was more than one font in {}", args.font_path.to_string_lossy());
		exit(2);
	}

	let font = fonts.remove(0);

	let glyph_c = font.glyph_count();
	println!("There are {} glyphs in {}", glyph_c, args.font_path.to_string_lossy());
}
