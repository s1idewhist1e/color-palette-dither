use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};

pub mod color_spaces;
use color_spaces::*;
use itertools::Itertools;

use rayon::prelude::*;

#[derive(Default)]
pub struct DitherBuilder {
    dimensions: Option<(u32, u32)>,
}

impl DitherBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn dimensions(mut self, dimensions: (u32, u32)) -> Self {
        self.dimensions = Some(dimensions);
        self
    }
    pub fn ordered_dither(
        self,
        img: DynamicImage,
        palette: &[impl Color + Copy + Sync + Send],
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        const N: u8 = 5; // log_2(side_length)
        let matrix = ThresholdMatrix::bayer_matrix(N);

        let (width, height) = img.dimensions();

        let img_vec = img
            .as_rgb8()
            .unwrap()
            .enumerate_pixels()
            // .iter_mut()
            .flat_map(move |(x, y, assign_color)| {
                // println!("Running pixel {},{}", x, y);
                let color = SRGB::from(assign_color as &image::Rgb<u8>).lab();
                // https://bisqwit.iki.fi/story/howto/dither/jy/
                let factor = matrix.get(x % matrix.x as u32, y % matrix.y as u32);
                let mut plan_color1 = palette.first().unwrap();
                let mut plan_color2 = palette.get(1).unwrap();
                let mut plan_ratio = 0.5;
                let mut penalty = f32::MAX;
                for i in 0..palette.len() {
                    for j in i + 1..palette.len() {
                        let color1 = palette.get(i).unwrap();
                        let color2 = palette.get(j).unwrap();

                        let (local_penalty, ratio) = evaluate_distance(&color, color1, color2);

                        if local_penalty < penalty {
                            penalty = local_penalty;
                            plan_ratio = ratio;
                            plan_color1 = color1;
                            plan_color2 = color2;
                        }
                    }
                }

                // **assign_color = lerp_color(plan_ratio, plan_color1, plan_color2).srgb().into()
                //
                // **assign_color = Rgb([(factor * 256.) as u8; 3]);
                // **assign_color = plan_color1.srgb().into();

                if factor > plan_ratio {
                    // **assign_color = Rgb([0,0,255])
                    Rgb::from(plan_color1.srgb()).0
                } else {
                    // **assign_color = Rgb([0,255,0])
                    Rgb::from(plan_color2.srgb()).0
                    // let c = plan_color2.srgb();
                    // [c.r, c.g, c.b]
                }
            })
            .collect_vec();

        ImageBuffer::from_vec(width, height, img_vec).unwrap()
    }
}

fn euclidean_distance_sq(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
    (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1) + (a.2 - b.2) * (a.2 - b.2)
}

/// returns (error, ratio)
fn evaluate_distance(color: &dyn Color, color1: &dyn Color, color2: &dyn Color) -> (f32, f32) {
    let color = color.lab();
    let color1 = color1.lab();
    let color2 = color2.lab();
    let a = (
        color2.l - color1.l,
        color2.a - color1.a,
        color2.b - color1.b,
    );

    let b = (color.l - color1.l, color.a - color1.a, color.b - color1.b);

    // let color = color.srgb();
    // let color1 = color1.srgb();
    // let color2 = color2.srgb();
    //
    // let a = (
    //     color2.r - color1.r,
    //     color2.g - color1.g,
    //     color2.b - color1.b,
    // );
    // let b = (color.r - color1.r, color.g - color1.g, color.b - color1.b);
    let dot = a.0 * b.0 + a.1 * b.1 + a.2 * b.2;

    let mag_a_sq = a.0 * a.0 + a.1 * a.1 + a.2 * a.2;

    // Short circuit on the edge case that `color1` and `color2` are equals
    const EPSILON: f32 = 1e-6;
    if mag_a_sq < EPSILON {
        return (0., (b.0 * b.0 + b.1 * b.1 + b.2 * b.2));
    }

    // Find the component of the line segment from color1->color2 that is closest to color
    let ratio = (dot / mag_a_sq).clamp(0., 1.);

    // scale the vector color1->color2 to ratio
    let closest_point = (a.0 * ratio, a.1 * ratio, a.2 * ratio);

    // Find the distance between these two
    let err = euclidean_distance_sq(closest_point, b);

    let err = err
        + 0.05
            * euclidean_distance_sq(
                (color1.l, color1.a, color1.b),
                (color2.l, color2.a, color2.b),
            );

    // dbg!(err, ratio);
    assert!(err >= 0.);
    assert!(ratio >= 0.);
    assert!(ratio <= 1.);

    (err, ratio)
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

                v as f32 / (side_length * side_length) as f32
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
