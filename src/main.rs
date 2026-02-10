use clap::Parser;
use std::path::Path;

use color_palette_dither::color_spaces::*;

use itertools::Itertools;

use image::{DynamicImage, Rgb};

mod argument_parsing;

use color_palette_dither::DitherBuilder;

const HELPTEXT: &'static str = "
A dithering tool that does ordered dithering with an arbitrary color palette

Usage:
color-palette-dither <input> <output> <palette>
where:
    input is the path to the input file to read from
    output is the path to the file to output to
    palette is the path to read the palette image from
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = argument_parsing::Args::parse();

    let output_file = args.output_file.unwrap_or("out.png".into());

    let source_path = Path::new(&args.input_file);
    let dest_path = Path::new(&output_file);
    let palette_path = Path::new(&args.palette_file);

    let palette = get_palette(palette_path)?;

    let img = image::ImageReader::open(source_path)?.decode()?;

    // resize image
    let in_width = 1920;
    let in_height = 1080;

    let mut width = in_width;
    let mut height = in_height;

    let input_aspect = img.width() as f32 / img.height() as f32;
    let output_aspect = width as f32 / height as f32;

    if input_aspect < output_aspect {
        // limiting factor is width
        height = (width as f32 / input_aspect) as u32;
    } else {
        width = (height as f32 * input_aspect) as u32;
    }

    let img = img.resize(width, height, image::imageops::FilterType::CatmullRom);

    let img = img.crop_imm(
        (width - in_width) / 2,
        (height - in_height) / 2,
        in_width,
        in_height,
    );

    let output = DitherBuilder::new().ordered_dither(img, &palette);

    output.save(Path::new(dest_path))?;

    // TODO: (maybe) implement irregular pallets
    //
    // let mut pallet_l_sorted = pallet.iter().map(|v| v.l).collect_vec();
    // palette.sort_by(|a,b| a.l.total_cmp(&b.l));
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
        .unique()
        .map(|pixel| SRGB::from(pixel).lab())
        // .map(|pixel| Color::from(pixel).into_cielab())
        .collect())
}

// fn color_error(
//     color: impl Color,
//     mixed: impl Color,
//     color1: impl Color,
//     color2: impl Color,
//     ratio: f32,
// ) -> f32 {
//     let color = color.lab();
//     let mixed = mixed.lab();
//     let color1 = color1.lab();
//     let color2 = color2.lab();
//     euclidean_distance_squared(color, mixed) + 0.1 * euclidean_distance_squared(color1, color2)
// }
