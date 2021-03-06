use image::{ImageBuffer, Rgb};
use std::fs::{create_dir, File};
use std::io::Read;
use std::path::Path;
use std::process::exit;
use rusttype::{Font, Glyph, FontCollection, Point, Rect, Scale, SharedBytes};

pub fn load_glyphs(font_path: &Path, chars_to_render: &Vec<char>) -> Vec<(char, Glyph<'static>)> {
    // First we read in the file into a byte buffer
    let mut font_file = File::open(font_path).unwrap();
    let mut byte_buffer = Vec::new();
    if let Err(read_error) = font_file.read_to_end(&mut byte_buffer) {
        println!("There was an error reading from {}\n{}",
                 font_path.to_string_lossy(),
                 read_error);
        exit(2);
    }

    // Then we convert that byte buffer into a Vec of Fonts
    let mut fonts_in_file: Vec<Font> = FontCollection::from_bytes(SharedBytes::ByRef(&byte_buffer))
        .into_fonts()
        .collect();

    // For now we can only handle having one font in the file
    if fonts_in_file.len() == 0 {
        println!("There were no fonts in {}", font_path.to_string_lossy());
        exit(2);
    }
    if fonts_in_file.len() > 1 {
        println!("There was more than one font in {}",
                 font_path.to_string_lossy());
        exit(2);
    }
    let font = fonts_in_file.remove(0);

    // Now we extract the glyphs for the characters we want to render with
    // We emit a warning if we couldn't extract a glyph for a character
    let mut glyphs = Vec::new();
    for c in chars_to_render {
        let result = font.glyph(c.clone());

        let glyph = match result {
            Some(glyph) => glyph,
            None => {
                println!("WARN: There was an error loading the glyph for {}", c);
                continue;
            }
        };

        glyphs.push((c.clone(), glyph.standalone()));
    }

    glyphs
}

pub fn render_glyphs(
    glyphs: &Vec<(char, Glyph)>,
    renders: &mut Vec<GlyphRender>,
    background: [u8; 3],
    foreground: [u8; 3],
    height: u32,
    ratio: f32
) {
    let width = (height as f32 / ratio) as u32;

    // Now we transform the glyphs to GlyphRenders (fancy image buffers)
    for &(ref c, ref glyph) in glyphs {
        // The Glyph needs scale and position information
        let positioned_glyph = glyph.standalone()
            .scaled(Scale { x: 40.0, y: 80.0 })
            .positioned(Point { x: 0.0, y: 0.0 });

        // The renderer needs information about the scaled glyph
        let mut renderer = GlyphRenderer::new(
            positioned_glyph.pixel_bounding_box().unwrap(),
            Rgb { data: background },
            Rgb { data: foreground },
            height,
            width,
            c.clone()
        );

        // Now draw it and push the result
        positioned_glyph.draw(&mut renderer);
        renders.push(renderer.finalize());
    }

}

pub fn export_glyph_renders(work_dir: &Path, glyph_renders: &Vec<GlyphRender>) {
    let mut index = 0;
    let render_dir = work_dir.join("glyph_renders");

    if let Err(error) = create_dir(&render_dir) {
        println!("There was an error making the render dir {}:\n{}",
                 render_dir.to_string_lossy(),
                 error);
        exit(3);
    }

    for render in glyph_renders {
        let render_path = render_dir.join(format!("{}.png", index));
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
    x_offset: u32,
    y_offset: u32,
    c: char,
}

impl FnOnce<(u32, u32, f32)> for GlyphRenderer {
    type Output = ();
    extern "rust-call" fn call_once(self, _: (u32, u32, f32)) -> () {}
}

impl FnMut<(u32, u32, f32)> for GlyphRenderer {
    extern "rust-call" fn call_mut(&mut self, args: (u32, u32, f32)) -> () {
        let t_r = (self.foreground_f[0] * args.2 + self.background_f[0] * (1.0 - args.2)) as u8;
        let t_g = (self.foreground_f[1] * args.2 + self.background_f[1] * (1.0 - args.2)) as u8;
        let t_b = (self.foreground_f[2] * args.2 + self.background_f[2] * (1.0 - args.2)) as u8;

        self.buffer.put_pixel(args.0 + self.x_offset,
                              args.1 + self.y_offset,
                              Rgb { data: [t_r, t_g, t_b] });
    }
}

impl GlyphRenderer {
    fn new(bounding_box: Rect<i32>,
           background: Rgb<u8>,
           foreground: Rgb<u8>,
           height: u32,
           width: u32,
           c: char)
           -> GlyphRenderer {
        let background_f = [background[0].clone() as f32,
                            background[1].clone() as f32,
                            background[2].clone() as f32];

        let foreground_f = [foreground[0].clone() as f32,
                            foreground[1].clone() as f32,
                            foreground[2].clone() as f32];

        let buffer = ImageBuffer::from_pixel(width, height, background);

        let bb_width = (bounding_box.max.x - bounding_box.min.x) as u32;
        let bb_height = (bounding_box.max.y - bounding_box.min.y) as u32;

        let x_offset = (width - bb_width) / 2;
        let y_offset = (height - bb_height) / 2;

        GlyphRenderer {
            buffer: buffer,
            background_f: background_f,
            foreground_f: foreground_f,
            background: background,
            foreground: foreground,
            x_offset: x_offset,
            y_offset: y_offset,
            c: c,
        }
    }

    fn finalize(self) -> GlyphRender {
        GlyphRender {
            buffer: self.buffer,
            background: self.background,
            foreground: self.foreground,
            c: self.c,
        }
    }
}

pub struct GlyphRender {
    pub buffer: ImageBuffer<Rgb<u8>, Vec<u8>>,
    background: Rgb<u8>,
    foreground: Rgb<u8>,
    c: char,
}

impl GlyphRender {
    fn export(&self, path: &Path) {
        let result = self.buffer.save(path);

        if let Err(error) = result {
            println!("There was an error saving the GlyphRender for {}:\n{}",
                     self.c,
                     error);
            exit(3);
        }
    }
}
