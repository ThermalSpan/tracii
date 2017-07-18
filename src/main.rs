#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate clap;
extern crate glob;
extern crate image;
#[macro_use] extern crate maplit;
extern crate rand;
extern crate rusttype;
extern crate tempdir;

mod args_and_usage;
mod render_glyphs;
mod xterm_colors;
mod image_util;

use std::process::exit;

fn main() {
    // If we cannot build args, the program will perform a non-zero exit
    let args = args_and_usage::parse_args();
    println!("Work dir: {}", args.work_dir.to_string_lossy());

    
    let chars;
    if args.limited_char_set {
        chars = (45..50)
            .map(|i| From::from(i))
            .collect();
    } else {
        chars = (33..127)
            .map(|i| From::from(i))
            .collect();
    }

    let glyph_pairs = render_glyphs::load_glyphs(&args.font_path, &chars);

    let mut renders = Vec::new();
   
    println!("Which color?");
    if args.color_256 {
        println!("256");
        let color_map = xterm_colors::make_xterm_color_map();

        'background_loop: for b in 0..255 {
            println!("b: {}", b);
            'foreground_loop: for f in 0..255 {
                if b == f {
                    continue 'foreground_loop;
                }

                if b > 56 {
                    break 'background_loop;
                }

                if f > 56 {
                    break 'foreground_loop;
                }

                let background = color_map.get(&b).unwrap().clone();
                let foreground = color_map.get(&f).unwrap().clone();

                render_glyphs::render_glyphs(
                    &glyph_pairs,
                    &mut renders,
                    background,
                    foreground,
                    80,
                    args.cell_ratio
                )
            }
        }
    } 
    
    else {
        println!("Boring color");
        render_glyphs::render_glyphs(
            &glyph_pairs,
            &mut renders,
            [240, 40, 14],
            [9, 200, 220],
            80,
            args.cell_ratio
        );
    }

    if args.export_glyph_renders {
        render_glyphs::export_glyph_renders(&args.work_dir, &renders);
    }

    if let Some(b) = image_util::pane_scramble (
        &renders.iter().map(|render| &render.buffer).collect(),
        [0, 0, 0],
        10,
        5
    ) {
        let path = args.work_dir.join("scramble.png");
        let result = b.save(path);

        if let Err(error) = result {
            println!("There was an error saving the render scramble:\n{}",
                error);
            exit(3);
        }

    }

    println!("Work dir: {}", args.work_dir.to_string_lossy());
}
 
