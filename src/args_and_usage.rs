use clap::{Arg, ArgGroup, App};
use glob::glob;
use std::path::PathBuf;
use std::process::exit;

// Programmer defined constants
static PROGRAM_NAME: &'static str = "tracii";

// Derived constants
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

// Font directories - places to check
// https://support.apple.com/en-us/HT201722
static FONT_DIRECTORIES: &'static [&str] = &[
    "~/Library/Fonts/", "/Library/Fonts/", "/Network/Library/Fonts/",
    "/System/Library/Fonts/", "/System Folder/Fonts/"
];

pub struct Args {
    pub cell_ratio: f32,
    pub font_path: PathBuf
}

pub fn parse_args() -> Args {
    let args = App::new(PROGRAM_NAME)
        .version(VERSION)
        .author("Russell W. Bentley <russell.w.bentley@icloud.com>")
        .about("A tool for generating fancy ASCII art")
        .arg(Arg::with_name("FONT_FILE")
            .help("Font file to use for rendering")
            .long("fontfile")
            .value_name("font/path")
            .takes_value(true))
        .arg(Arg::with_name("FONT_NAME")
            .help("The name of a font to use")
            .long("fontname")
            .value_name("name")
            .takes_value(true))
        .group(ArgGroup::with_name("FONT")
            .arg("FONT_FILE")
            .arg("FONT_NAME")
            .required(true))
        .arg(Arg::with_name("RATIO")
            .help("The height to width ratio of a glyph cell")
            .long("cellratio")
            .value_name("h/w")
            .takes_value(true))
        .get_matches();

    // The cell_ratio is a float parsed from a str with a default of 1.9
    // exit on a parse error
    let cell_ratio = match args.value_of("RATIO") {
        Some(ratio_str) => {
            match ratio_str.parse() {
                Ok(ratio) => ratio,
                Err(parse_error) => {
                    println!("--cellratio / -r must be parsable as an f32");
                    println!("Attempting to parse {} gave the following error:\n{}", ratio_str, parse_error);
                    println!("\n{}", args.usage());
                    exit(1)
                }
            }
        },
        None => 1.9f32
    };

    // We are either passed a name or a file
    let font_path = match (args.value_of("FONT_FILE"), args.value_of("FONT_NAME")) {
        (Some(file_path_str), None) => {
            let path = PathBuf::from(file_path_str);
            if ! path.exists() {
                println!("{} does not exist", path.to_string_lossy());
                exit(1);
            }
            path
        }
        (None, Some(font_name_str)) => find_font(font_name_str),
        _ => {
            println!("Either both --fontfile and --fontname were passed or neither.");
            println!("It shouldn't be possible to see this! File a bug!");
            exit(1)
        }
    };

    Args {
        cell_ratio: cell_ratio,
        font_path: font_path,
    }
}

fn find_font(name: &str) -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();

    for directory in FONT_DIRECTORIES {
        let pattern = String::from(*directory) + name + "*";
        let paths = match glob(&pattern) {
            Ok(paths) => paths,
            Err(pattern_error) => {
                println!("There was a pattern error while searching for the font name {}:", name);
                println!("{}", pattern_error);
                exit(1)
            }
        };

        for path in paths {
            match path {
                Ok(path) => candidates.push(path),
                Err(glob_error) => {
                    println!("There was a glob error:\n{}", glob_error);
                    exit(1);
                }
            }
        }
    }

    if candidates.len() == 0 {
        println!("Unable to locate a font with the name {}", name);
        exit(1);
    }

    if candidates.len() > 1 {
        println!("We found the the following font files that matched {}:", name);
        for candidate in candidates {
            println!("\t{}", candidate.to_string_lossy());
        }
        println!("\nThere can only be one viable file");
        exit(1);
    }

    candidates.remove(0)
}
