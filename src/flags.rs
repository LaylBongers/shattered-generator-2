use std::fs;
use imagefmt::{self, ColFmt, ColType};
use palette::{Rgb, Rgba};
use palette::blend::PreAlpha;
use rand::{Rng, StdRng};
use config::Config;
use Eu4TargetData;

pub fn generate(config: &Config, data: &Eu4TargetData) {
    println!("Generating flags...");

    // Set up the directory to output flags to
    let mut flag_base = config.target_path.clone();
    flag_base.push("gfx");
    flag_base.push("flags");
    fs::create_dir_all(&flag_base).unwrap();

    // Go over all requested flags
    let mut rand = StdRng::new().unwrap();
    for flag in &data.flag_requests {
        let mut flag_file = flag_base.clone();
        flag_file.push(format!("{}.tga", flag.tag));

        // Prepare some data to generate this flag
        let flag_func = get_flag_function(&mut rand);

        // Generate the image
        let width = 128;
        let area_per_pixel = 1.0 / width as f32;
        let size = width*width;
        let per_pixel = 3;
        let mut buffer = vec![0u8; size*per_pixel];
        for yi in 0..width {
            for xi in 0..width {
                // Calculate normalized coordinates
                let x = (xi as f32) / (width as f32);
                let y = (yi as f32) / (width as f32);

                // Calculate this pixel's color
                let color = flag_func(x, y, area_per_pixel, flag.color, flag.color_alt);

                // Calculate and store the color for this pixel
                let u8_color: [u8; 3] = color.to_pixel();
                let actual = ((yi*width)+xi)*per_pixel;
                buffer[actual+0] = u8_color[0];
                buffer[actual+1] = u8_color[1];
                buffer[actual+2] = u8_color[2];
            }
        }

        // Write the image to a file
        imagefmt::write(
            &flag_file,
            128, 128, ColFmt::RGB,
            &buffer,
            ColType::Color
        ).unwrap();
    }
}

fn get_flag_function(rand: &mut StdRng) -> Box<Fn(f32, f32, f32, Rgb, Rgb) -> Rgba> {
    let num: i32 = rand.gen_range(0, 6);
    match num {
        0 => Box::new(func_flat_flag),
        1 => Box::new(func_dashed_flag),
        2 => Box::new(func_dashed_inverted_flag),
        3 => Box::new(func_crossed_flag),
        4 => Box::new(func_horizontal_line_flag),
        5 => Box::new(func_vertical_line_flag),
        _ => panic!("Generated flag type out of range")
    }
}

fn func_flat_flag(_x: f32, _y: f32, _area_per_pixel: f32, color: Rgb, _color_alt: Rgb) -> Rgba {
    flag_shader_flat(color)
}

fn func_dashed_flag(x: f32, y: f32, area_per_pixel: f32, color: Rgb, color_alt: Rgb) -> Rgba {
    msaa(x, y, area_per_pixel, |x, y| {
        let base = flag_shader_flat(color);
        let overlay = flag_shader_diagonal(x, y, color_alt);
        blend(PreAlpha::from(overlay), base)
    })
}

fn func_dashed_inverted_flag(x: f32, y: f32, area_per_pixel: f32, color: Rgb, color_alt: Rgb) -> Rgba {
    func_dashed_flag(1.0-x, y, area_per_pixel, color, color_alt)
}

fn func_crossed_flag(x: f32, y: f32, area_per_pixel: f32, color: Rgb, color_alt: Rgb) -> Rgba {
    msaa(x, y, area_per_pixel, |x, y| {
        let base = flag_shader_flat(color);
        let overlay1 = flag_shader_diagonal(x, y, color_alt);
        let overlay2 = flag_shader_diagonal(1.0-x, y, color_alt);
        blend(PreAlpha::from(overlay2), blend(PreAlpha::from(overlay1), base))
    })
}

fn func_horizontal_line_flag(_x: f32, y: f32, _area_per_pixel: f32, color: Rgb, color_alt: Rgb) -> Rgba {
    let base = flag_shader_flat(color);
    let overlay = flag_shader_middle_line(y, color_alt);
    blend(PreAlpha::from(overlay), base)
}

fn func_vertical_line_flag(x: f32, _y: f32, _area_per_pixel: f32, color: Rgb, color_alt: Rgb) -> Rgba {
    let base = flag_shader_flat(color);
    let overlay = flag_shader_middle_line(x, color_alt);
    blend(PreAlpha::from(overlay), base)
}

/// Blends source onto dest, source has to be pre-multiplied.
/// TODO: Make use of palette's own blend function
fn blend(source: PreAlpha<Rgb, f32>, dest: Rgba) -> Rgba {
    let one_minus_alpha = 1.0 - source.alpha;

    let r = source.red + (dest.red * one_minus_alpha);
    let g = source.green + (dest.green * one_minus_alpha);
    let b = source.blue + (dest.blue * one_minus_alpha);

    Rgba::new(r, g, b, 1.0)
}

/// Multisample the given shader function.
fn msaa<F: Fn(f32, f32) -> Rgba>(x: f32, y: f32, area: f32, func: F) -> Rgba {
    let per = area / 4.0;
    let c0 = func(x + per, y + per);
    let c1 = func(x + per*3.0, y + per);
    let c2 = func(x + per, y + per*3.0);
    let c3 = func(x + per*3.0, y + per*3.0);

    Rgba::new(
        average(c0.red, c1.red, c2.red, c3.red),
        average(c0.green, c1.green, c2.green, c3.green),
        average(c0.blue, c1.blue, c2.blue, c3.blue),
        1.0
    )
}

fn average(v0: f32, v1: f32, v2: f32, v3: f32) -> f32 {
    v0*0.25 + v1*0.25 + v2*0.25 + v3*0.25
}

fn flag_shader_flat(color: Rgb) -> Rgba {
    Rgba::new(color.red, color.green, color.blue, 1.0)
}

fn flag_shader_diagonal(x: f32, y: f32, color: Rgb) -> Rgba {
    if f32::abs(x - y) < 0.15 {
        Rgba::new(color.red, color.green, color.blue, 1.0)
    } else {
        Rgba::new(0.0, 0.0, 0.0, 0.0)
    }
}

fn flag_shader_middle_line(axis: f32, color: Rgb) -> Rgba {
    if axis > 0.35 && axis < 0.65 {
        Rgba::new(color.red, color.green, color.blue, 1.0)
    } else {
        Rgba::new(0.0, 0.0, 0.0, 0.0)
    }
}
