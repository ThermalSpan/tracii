use image::{GenericImage, ImageBuffer, Rgb, RgbImage};
use rand::{thread_rng, Rng};

pub fn pane_scramble (
    buffers: &Vec<&RgbImage>,
    background: [u8; 3],
    width_count: usize,
    height_count: usize,
) -> Option<RgbImage>
{
    if buffers.len() == 0 {
        return None;
    }
    
    let mut rand = thread_rng();
    let mut panes: Vec<&&RgbImage> = buffers.iter().take(width_count * height_count).collect();
    for i in 0..panes.len() {
        let j = rand.next_u32() as usize % (panes.len() - 1);
        if i == j {
            continue;
        }

        let temp = panes[i];
        panes[i] = panes[j];
        panes[j] = temp;
    }

    let offering = buffers[0];
    let width = width_count * offering.width() as usize;
    let height = height_count * offering.height() as usize;

    let mut result = ImageBuffer::from_pixel(
        width  as u32,
        height as u32,
        Rgb{ data: background}
    );
    
    'outer_loop: for w in 0..width_count {
        for h in 0..height_count {
            let pane = match buffers.get(width + height * width_count as usize) {
                Some(pane) => pane,
                None => break 'outer_loop,
            };

            if pane.width() as usize != width {
                println!(
                    "ERROR: the offering pane had width of {}, found a pane with width of {}", 
                    width, 
                    pane.width()
                );
                return None;
            }

            if pane.height() as usize != height {
                println!(
                    "ERROR: the offering pane had height of {}, found a pane with height of {}", 
                    height, 
                    pane.height()
                );
                return None;
            }

            result.copy_from(
                *pane,
                (w * width) as u32,
                (h * height) as u32,
            );
        }
    }

    Some(result)
}

