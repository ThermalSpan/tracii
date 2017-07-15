#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate clap;
extern crate glob;
extern crate image;
extern crate rusttype;
extern crate tempdir;

mod args_and_usage;
mod render_glyphs;

fn main() {
    // If we cannot build args, the program will perform a non-zero exit
    let args = args_and_usage::parse_args();

    let standard_characters: Vec<char> = (33..127)
        .map(|i| From::from(i))
        .collect();

    let glyph_pairs = render_glyphs::load_glyphs(&args.font_path, &standard_characters);

    let glyph_renders = render_glyphs::render_glyphs(&glyph_pairs,
                                                     [240, 40, 14],
                                                     [9, 200, 220],
                                                     80,
                                                     args.cell_ratio);

    if args.export_glyph_renders {
        render_glyphs::export_glyph_renders(&args.work_dir, &glyph_renders);
    }

    println!("Work dir: {}", args.work_dir.to_string_lossy());
}
