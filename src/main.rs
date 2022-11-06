#[macro_use] extern crate log;
extern crate simplelog;

mod padding;

use clap::{command, Arg, ArgAction, ArgMatches};
use image::{DynamicImage, GenericImageView, ImageError, io::Reader};
use log::LevelFilter;
use padding::Padding;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TerminalMode, TermLogger};
// use std::path::Path;

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

fn clean_and_optimize(image: &mut DynamicImage) {
    let empty = Padding::new(
        count_top_empty_rows(image),
        count_bottom_empty_rows(image),
        count_left_empty_columns(image),
        count_right_empty_columns(image)
    );
    debug!("empty: {:?}", empty);

    let mut padding = Padding::new(0, 0, 0, 0);
    info!("Current image resolution: ({}, {})", image.width(), image.height());

    if empty.left < 4 && empty.right < 4 {
        let remainder = (image.width() - empty.left - empty.right) % 4;
        debug!("remainder: {}", remainder);
        let additional_padding = 4 - remainder;
        match additional_padding {
            1 => { padding.left += 1; },
            2 => {
                padding.left += 1;
                padding.right += 1;
            },
            3 => {
                padding.left += 1;
                padding.right += 2;
            },
            4 => {
                padding.left += if empty.left == 0 { 1 } else { 0 };
                padding.right += if empty.right == 0 { 1 } else { 0 };
            },
            0 | 5..=u32::MAX => { }
        }
        debug!("padding: {:?}", padding);
    }

    if empty.top < 4 && empty.bottom < 4 {
        let remainder = (image.height() - empty.top - empty.bottom) % 4;
        debug!("remainder: {}", remainder);
        let additional_padding = 4 - remainder;
        debug!("additional padding: {}", additional_padding);
        match additional_padding {
            1 => { padding.bottom += 1; },
            2 => {
                padding.top += 1;
                padding.bottom += 1;
            },
            3 => {
                padding.top += 1;
                padding.bottom += 2;
            },
            4 => {
                padding.top += if empty.top == 0 { 1 } else { 0 };
                padding.bottom += if empty.bottom == 0 { 1 } else { 0 };
            },
            0 | 5..=u32::MAX => { }
        }
        debug!("padding: {:?}", padding);
    }

    let target_width = image.width() - empty.left - empty.right + padding.left + padding.right;
    let target_height = image.height() - empty.top - empty.bottom + padding.top + padding.bottom;
    info!("Target resolution: ({}, {})", target_width, target_height);
}

fn handle_error(error: &str) {
    error!("{}", error);
    std::process::exit(1);
}

fn main() {
    let args = parse_arguments();
    initialize_logger();

    let input_path = args.get_one::<String>("input").unwrap();
    // let output_path = args.get_one::<String>("output").unwrap();
    // let force = args.get_flag("force");

    // let output_is_file = Path::new(output_path).is_file();

    let mut image: DynamicImage = match try_read_image(&input_path) {
        Err(e) => {
            let error_message = format!("Error while reading image: {}, {}", input_path, e);
            handle_error(error_message.as_str());
            return;
        },
        Ok(img) => { img }
    };

    clean_and_optimize(&mut image);

    // if output_is_file && !force {
    //     handle_error("Output already exists. Aborting. Specify --force flag if you want to replace the output");
    // }
    //
    // match image.save(output_path) {
    //     Err(e) => {
    //         handle_error(e.to_string().as_str());
    //         return;
    //     },
    //     Ok(_) => {
    //         info!("Successfully exported to: {}", output_path);
    //     }
    // }
}
