use std::{env, f32::consts::PI, path::Path};

use color_spaces::*;

use rayon::prelude::*;

use itertools::Itertools;

use image::{DynamicImage, Rgb};

mod color_spaces;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Determine median distance between palette colors

    let args = env::args().collect_vec();
    let source_path = Path::new(&args[1]);
    let dest_path = Path::new(&args[2]);
    let palette_path = Path::new(&args[3]);

    let palette = get_palette(palette_path)?;

    let img = image::ImageReader::open(source_path)?.decode()?;

    let output = ordered_dither(img, &palette);

    output.save(Path::new(dest_path))?;

    // TODO: (maybe) implement irregular pallets
    //
    // let mut pallet_l_sorted = pallet.iter().map(|v| v.l).collect_vec();
    // pallet_l_sorted.sort_by(f32::total_cmp);
    // let mut pallet_l_difference = pallet_l_sorted
    //     .iter()
    //     .tuple_windows::<(&f32, &f32)>()
    //     .map(|(a, b)| b - a)
    //     .collect_vec();
    //
    // // determine median
    // pallet_l_difference.sort_by(f32::total_cmp);
    // let l = if pallet_l_difference.len() % 2 == 0 {
    //     // get average of two middle indices
    //     (pallet_l_difference
    //         .get(pallet_l_difference.len() / 2)
    //         .unwrap()
    //         + pallet_l_difference
    //             .get(pallet_l_difference.len() / 2 - 1)
    //             .unwrap())
    //         / 2.
    // } else {
    //     // middle index
    //     *pallet_l_difference
    //         .get(pallet_l_difference.len() / 2 )
    //         .unwrap()
    // };

    // for i in get_palette(std::path::Path::new("../pallete.lab.png")).unwrap() {
    //     // let b = i.into_srgb();
    //     // if let Color::Srgb { r, g, b } = b {
    //     //     println!("{}, {}, {}", r, g, b);
    //     // }
    //     let a = i.lab();
    //         println!("{}, {}, {}", a.l, a.a, a.b);
    //     let c = a.xyz();
    //         println!("{}, {}, {}", c.x, c.y, c.z);
    //     let d = c.srgb();
    //         println!("{}, {}, {}\n", d.r, d.g, d.b);
    // }
    //
    Ok(())
}

fn get_palette(palette: &std::path::Path) -> std::result::Result<Vec<LAB>, image::ImageError> {
    dbg!("{}", palette);
    let mut img = image::ImageReader::open(palette)?.decode()?;
    img.apply_color_space(
        image::metadata::Cicp::SRGB,
        image::ConvertColorOptions::default(),
    )?;
    Ok(img
        .into_rgb8()
        .pixels()
        .map(|pixel| SRGB::from(pixel).lab())
        // .map(|pixel| Color::from(pixel).into_cielab())
        .collect())
}

// TODO: Implement arbitrary threshold matrix
fn ordered_dither(mut img: DynamicImage, palette: &[impl Color + Copy + Sync + Send]) -> DynamicImage {
    const N: u8 = 5; // log_2(side_length)
    let matrix = ThresholdMatrix::bayer_matrix(N);

    img.as_mut_rgb8()
        .unwrap()
        .enumerate_pixels_mut()
        .collect_vec().par_iter_mut()
        .for_each(move|(x, y, assign_color)| {
            // println!("Running pixel {},{}", x, y);
            let color = SRGB::from(assign_color as &image::Rgb<u8>).lab();
            // https://bisqwit.iki.fi/story/howto/dither/jy/
            let factor = matrix.get(*x % matrix.x as u32, *y % matrix.y as u32);
            let mut plan_color1 = palette.first().unwrap().lab();
            let mut plan_color2 = palette.get(1).unwrap().lab();
            let mut plan_ratio = 0.5;
            let mut penalty = f32::MAX;
            for i in 0..palette.len() {
                for j in i + 1..palette.len() {
                    let color1 = palette.get(i).unwrap().lab();
                    let color2 = palette.get(j).unwrap().lab();
                    for ratio in 0..matrix.x * matrix.y {
                        let ratio = ratio as f32 / (matrix.x * matrix.y) as f32;
                        let mixed = lerp_color(ratio, color1, color2);
                        let test_penalty = color_error(color, mixed, color1, color2, ratio);
                            // dbg!(test_penalty);
                        if penalty > test_penalty {
                            penalty = test_penalty;
                            plan_color1 = color1;
                            plan_color2 = color2;
                            plan_ratio = ratio;
                        }
                    }
                }
            }

            if factor < plan_ratio {
                **assign_color = plan_color1.srgb().into();
            } else {
                **assign_color = plan_color2.srgb().into();
            }
        });

    img
}

fn color_error(
    color: impl Color,
    mixed: impl Color,
    color1: impl Color,
    color2: impl Color,
    ratio: f32,
) -> f32 {
    let color = color.lab();
    let mixed = mixed.lab();
    let color1 = color1.lab();
    let color2 = color2.lab();
    euclidean_distance_squared(color, mixed)  + 0.33 * euclidean_distance_squared(color1, color2)
}

fn euclidean_distance_squared(a: LAB, b: LAB) -> f32 {
    // dbg!(a, b);
    ((a.l - b.l) * (a.l - b.l) + (a.a - b.a) * (a.a - b.a) + (a.b - b.b) * (a.b - b.b)).sqrt()
}

fn lerp_color(ratio: f32, color1: impl Color, color2: impl Color) -> LAB {
    let color1 = color1.lab();
    let color2 = color2.lab();

    LAB {
        l: (color1.l * (1. - ratio) + ratio * color2.l),
        a: (color1.a * (1. - ratio) + ratio * color2.a),
        b: (color1.b * (1. - ratio) + ratio * color2.b),
    }
}

fn closest_color<C: Color + Copy>(input: impl Color, palette: &[C]) -> C {
    // Closest by euclidean distance
    let input_lab = input.lab();
    *palette
        .iter()
        .map(|color| {
            let color_lab = color.lab();
            (
                (color_lab.l - input_lab.l).powi(2)
                    + (color_lab.a - input_lab.a).powi(2)
                    + (color_lab.b - input_lab.b).powi(2),
                color,
            )
        })
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap()
        .1
}

struct ThresholdMatrix {
    x: usize,
    y: usize,
    matrix: Vec<f32>,
}

impl ThresholdMatrix {
    fn bayer_matrix(side_length_order: u8) -> Self {
        let side_length = 2_usize.pow(side_length_order as u32);
        let matrix = (0..side_length)
            .cartesian_product(0..side_length)
            .map(|(x, y)| {
                let xor = x ^ y;

                let mut v = 0;
                for p in 0..side_length_order {
                    let bit_idx = 2 * (side_length_order - p - 1);
                    v |= ((y >> p) & 1) << bit_idx;
                    v |= ((xor >> p) & 1) << (bit_idx + 1);
                }

                let v = (v as f32 / (side_length * side_length) as f32);
                v
            })
            .collect_vec();

        dbg!(&matrix);

        Self {
            x: side_length,
            y: side_length,
            matrix,
        }
    }

    fn get(&self, x: u32, y: u32) -> f32 {
        let i = y as usize % self.y;
        let j = x as usize % self.x;
        *self.matrix.get(i * self.x + j).unwrap()
    }
}
