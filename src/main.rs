#[macro_use] extern crate log;
extern crate simplelog;

mod padding;

use clap::{command, Arg, ArgAction, ArgMatches};
use image::{DynamicImage, GenericImageView, ImageError, io::Reader, RgbaImage, Rgba};
use log::LevelFilter;
use padding::Padding;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TerminalMode, TermLogger};
use std::path::Path;

fn parse_arguments() -> ArgMatches {
    command!()
        .about("sprita helps prepare sprites for game development usage")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .required(true)
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .required(true)
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
        )
        .get_matches()
}

fn initialize_logger() {
    CombinedLogger::init(
        vec![
            TermLogger::new(
                LevelFilter::Trace,
                ConfigBuilder::new()
                    .set_time_level(LevelFilter::Debug)
                    .build(),
                TerminalMode::Mixed,
                ColorChoice::Auto
            )
        ]
    ).unwrap();
}


fn try_read_image(path: &str) -> Result<DynamicImage, ImageError> {
    Reader::open(path)?
        .with_guessed_format()?
        .decode()
}

fn is_row_empty(image: &DynamicImage, row: u32) -> bool {
    for x in 0..image.width() {
        if image.get_pixel(x, row).0[3] > 0 {
            return false;
        }
    }

    true
}

fn is_column_empty(image: &DynamicImage, column: u32) -> bool {
    for y in 0..image.height() {
        if image.get_pixel(column, y).0[3] > 0 {
            return false;
        }
    }

    true
}

fn count_top_empty_rows(image: &DynamicImage) -> u32 {
    let mut count = 0u32;
    for row in 0..image.height() {
        if !is_row_empty(image, row) {
            break;
        }
        count += 1;
    }

    count
}

fn count_bottom_empty_rows(image: &DynamicImage) -> u32 {
    let mut count = 0u32;
    for row in (0..image.height()).rev() {
        if !is_row_empty(image, row) {
            break;
        }
        count += 1;
    }

    count
}

fn count_left_empty_columns(image: &DynamicImage) -> u32 {
    let mut count = 0u32;
    for column in 0..image.width() {
        if !is_column_empty(image, column) {
            break;
        }
        count += 1;
    }

    count
}

fn count_right_empty_columns(image: &DynamicImage) -> u32 {
    let mut count = 0u32;
    for column in (0..image.width()).rev() {
        if !is_column_empty(image, column) {
            break;
        }
        count += 1;
    }

    count
}

fn count_empty_padding(image: &DynamicImage) -> Padding {
    Padding::new(
        count_top_empty_rows(image),
        count_bottom_empty_rows(image),
        count_left_empty_columns(image),
        count_right_empty_columns(image)
    )
}

fn is_image_good(image: &DynamicImage, empty: &Padding) -> bool {
    let is_dimension_optimized = image.width() % 4 == 0 && image.height() % 4 == 0;
    let empty_is_in_range = (1..4).contains(&empty.left) &&
        (1..4).contains(&empty.right) &&
        (1..4).contains(&empty.top) &&
        (1..4).contains(&empty.bottom);

    is_dimension_optimized && empty_is_in_range
}

fn clean(image: &DynamicImage, empty: &Padding) -> Option<DynamicImage> {
    if empty.left == 0 && empty.right == 0 && empty.top == 0 && empty.bottom == 0 {
        return None;
    }

    let target_width = image.width() - empty.left - empty.right;
    let target_height = image.height() - empty.top - empty.bottom;

    Some(image.crop_imm(empty.left, empty.top, target_width, target_height))
}

fn optimize(image: &DynamicImage, empty: &Padding) -> Option<DynamicImage> {
    let mut padding = Padding::new(0, 0, 0, 0);

    padding.left += if empty.left == 0 { 1 } else { 0 };
    padding.right += if empty.right == 0 { 1 } else { 0 };
    padding.top += if empty.top == 0 { 1 } else { 0 };
    padding.bottom += if empty.bottom == 0 { 1 } else { 0 };

    match (image.width() + padding.left + padding.right) % 4 {
        1 => {
            padding.left += 1;
            padding.right += 2;
        },
        2 => {
            padding.left += 1;
            padding.right += 1;
        },
        3 => { padding.right += 1; },
        0 | 4..=u32::MAX => { }
    }

    match (image.height() + padding.top + padding.bottom) % 4 {
        1 => {
            padding.top += 1;
            padding.bottom += 2;
        },
        2 => {
            padding.top += 1;
            padding.bottom += 1;
        },
        3 => { padding.bottom += 1; },
        0 | 4..=u32::MAX => { }
    }

    Some(add_padding(&image, &padding))
}

fn clean_and_optimize(image: &DynamicImage) -> Option<DynamicImage> {
    let empty = count_empty_padding(image);
    if is_image_good(image, &empty) {
        return None;
    }

    let cleaned_image = match clean(&image, &empty) {
        Some(img) => img,
        None => image.clone()
    };

    let empty = count_empty_padding(&cleaned_image);

    let optimized_image = match optimize(&cleaned_image, &empty) {
        Some(img) => img,
        None => cleaned_image
    };

    Some(optimized_image)
}

fn add_padding(image: &DynamicImage, padding: &Padding) -> DynamicImage {
    let width = image.width() + padding.left + padding.right;
    let height = image.height() + padding.top + padding.bottom;

    let mut image_buffer = RgbaImage::new(width, height);
    let right_index = padding.left + image.width();
    let bottom_index = padding.top + image.height();

    for column in 0..width {
        for row in 0..height {
            if column >= padding.left && column < right_index && row >= padding.top && row < bottom_index {
                image_buffer.put_pixel(column, row, image.get_pixel(column - padding.left, row - padding.top));
                continue;
            }
            
            image_buffer.put_pixel(column, row, Rgba([0, 0, 0, 0]));
        }
    }

    DynamicImage::from(image_buffer)
}

fn handle_error(error: &str) {
    error!("{}", error);
    std::process::exit(1);
}

fn main() {
    let args = parse_arguments();
    initialize_logger();

    let input_path = args.get_one::<String>("input").unwrap();
    let output_path = args.get_one::<String>("output").unwrap();
    let force = args.get_flag("force");

    let output_is_file = Path::new(output_path).is_file();

    let image: DynamicImage = match try_read_image(&input_path) {
        Err(e) => {
            let error_message = format!("Error while reading image: {}, {}", input_path, e);
            handle_error(error_message.as_str());
            return;
        },
        Ok(img) => { img }
    };

    let clean_image = match clean_and_optimize(&image) {
        Some(img) => img,
        None => image
    };

    if output_is_file && !force {
        handle_error("Output already exists. Aborting. Specify --force flag if you want to replace the output");
    }

    match clean_image.save(output_path) {
        Err(e) => {
            handle_error(e.to_string().as_str());
            return;
        },
        Ok(_) => {
            info!("Successfully exported to: {}", output_path);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn count_empty_clean_diamond() {
        let diamond_path = "./diamond_clean.png";
        let diamond_image = match try_read_image(&diamond_path) {
            Err(e) => {
                panic!("diamond image is unreadable, error: {}", e);
            },
            Ok(img) => {
                img
            }
        };
        let empty = Padding::new(
            count_top_empty_rows(&diamond_image),
            count_bottom_empty_rows(&diamond_image),
            count_left_empty_columns(&diamond_image),
            count_right_empty_columns(&diamond_image)
        );

        assert_eq!(empty.top, 0);
        assert_eq!(empty.left, 0);
        assert_eq!(empty.right, 0);
        assert_eq!(empty.bottom, 0);
    }

    #[test]
    fn clean_diamond() {
        let diamond_path = "./diamond_clean.png";
        let diamond_image = match try_read_image(&diamond_path) {
            Err(e) => {
                panic!("diamond image is unreadable, error: {}", e);
            },
            Ok(img) => {
                img
            }
        };
        let clean_image = match clean_and_optimize(&diamond_image) {
            Some(img) => img,
            None => diamond_image
        };

        assert_eq!(clean_image.width(), 248);
        assert_eq!(clean_image.height(), 208);
    }

    #[test]
    fn unclean_diamond() {
        let diamond_path = "./diamond.png";
        let diamond_image = match try_read_image(&diamond_path) {
            Err(e) => {
                panic!("diamond image is unreadable, error: {}", e);
            },
            Ok(img) => {
                img
            }
        };
        let clean_image = match clean_and_optimize(&diamond_image) {
            Some(img) => img,
            None => diamond_image
        };

        assert_eq!(clean_image.width(), 248);
        assert_eq!(clean_image.height(), 208);
    }
}
