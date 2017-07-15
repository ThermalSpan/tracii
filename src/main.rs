#![feature(unboxed_closures)]
#![feature(fn_traits)] 

extern crate clap;
extern crate glob;
extern crate image; 
extern crate rusttype;
extern crate tempdir;

mod args_and_usage;

use image::{ImageBuffer, Rgb};
use std::convert::From;
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
use std::process::exit;
use rusttype::{CodepointOrGlyphId, Font, FontCollection, Glyph, Point, Rect, Scale, SharedBytes};

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

    let standard_characters: Vec<char> = (33..127)
        .map(|i| From::from(i))
        .collect();
    
    let mut glyphs = Vec::new();
    for c in standard_characters {
        let result = font.glyph(c);

        let glyph = match result {
            Some(glyph) => glyph, 
            None => {
                println!("There was an error loading the glyph for {}", c);
                continue
            }
        };

        glyphs.push((c, glyph));
    }
    
    let mut glyph_renders = Vec::new();
    for (c, glyph) in glyphs {
        let positioned_glyph = glyph
            .scaled(Scale {x: 40.0, y: 80.0})
            .positioned(Point {x: 0.0, y: 0.0});

        let mut renderer = GlyphRenderer::new(
            positioned_glyph.pixel_bounding_box().unwrap(),
            Rgb {data: [255, 255, 255]},
            Rgb {data: [0, 0, 0]},
            c
        );

        positioned_glyph.draw(&mut renderer);
        
        glyph_renders.push(renderer.finalize());
    }
    
    export_glyph_renders(&args.work_dir, &glyph_renders);

	let glyph_c = font.glyph_count();
	println!("There are {} glyphs in {}", glyph_c, args.font_path.to_string_lossy());

	println!("Work dir: {}", args.work_dir.to_string_lossy());
}


fn export_glyph_renders(work_dir: &Path, glyph_renders: &Vec<GlyphRender>) {
    let mut index = 0;
    let render_dir = work_dir.join("glyph_renders");

	if let Err(error) = create_dir(&render_dir) {
		println!("There was an error making the render dir {}:\n{}", render_dir.to_string_lossy(), error);
		exit(3);
	}

    for render in glyph_renders {
        let render_path = render_dir.join(format!("{}-glyph-render.png", index));
        render.export(&render_path);
		index += 1;
    }
}

struct GlyphRenderer {
    buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
    background: Rgb<u8>,
    foreground: Rgb<u8>,
    background_f: [f32; 3],
    foreground_f: [f32; 3],
    c: char,
}
impl FnOnce<(u32, u32, f32)> for GlyphRenderer {
    type Output = ();
    extern "rust-call" fn call_once(self, args: (u32, u32, f32)) -> () {}
}

impl FnMut<(u32, u32, f32)> for GlyphRenderer {
    extern "rust-call" fn call_mut(&mut self, args: (u32, u32, f32)) -> () {
        let t_r = (self.foreground_f[0] * args.2 + self.background_f[0] * (1.0 - args.2)) as u8;
        let t_g = (self.foreground_f[1] * args.2 + self.background_f[1] * (1.0 - args.2)) as u8;
        let t_b = (self.foreground_f[2] * args.2 + self.background_f[2] * (1.0 - args.2)) as u8;
        
        self.buffer.put_pixel(args.0, args.1, Rgb {data: [t_r, t_g, t_b]});
    }
}

impl GlyphRenderer {
    fn new(
        bounding_box: Rect<i32>, 
        background: Rgb<u8>, 
        foreground: Rgb<u8>, 
        c: char
    ) -> GlyphRenderer 
    {
        let background_f = [
            background[0].clone() as f32,
            background[1].clone() as f32,
            background[2].clone() as f32
        ];
            
        let foreground_f = [
            foreground[0].clone() as f32,
            foreground[1].clone() as f32,
            foreground[2].clone() as f32
        ];

        let buffer = ImageBuffer::new(
            (bounding_box.max.x - bounding_box.min.x) as u32,
            (bounding_box.max.y - bounding_box.min.y) as u32,
        );

        GlyphRenderer {
            buffer: buffer,
            background_f: background_f,
            foreground_f: foreground_f,
            background: background,
            foreground: foreground,
            c: c
        }
    }

    fn finalize(self) -> GlyphRender {
        GlyphRender {
            buffer: self.buffer,
            background: self.background,
            foreground: self.foreground,
            c: self.c
        }
    }
}

struct GlyphRender {
    buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
    background: Rgb<u8>,
    foreground: Rgb<u8>,
    c: char,
}

impl GlyphRender {
    fn export(&self, path: &Path) {
        let result = self.buffer.save(path);
 
        if let Err(error) = result {
            println!("There was an error saving the GlyphRender for {}:\n{}", self.c, error);
            exit(3);
        }
    }
}
