#[macro_use] extern crate log;
extern crate simplelog;

mod padding;
mod ops;

use clap::Parser;
use image::{DynamicImage, ImageError, io::Reader};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TerminalMode, TermLogger};
use std::path::Path;

#[derive(Parser)]
struct Arguments {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    force: bool
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

fn handle_error(error: &str) {
    error!("{}", error);
    std::process::exit(1);
}

fn main() {
    let args = Arguments::parse();
    initialize_logger();

    let output_is_file = Path::new(&args.output).is_file();

    let image: DynamicImage = match try_read_image(&args.input) {
        Err(e) => {
            let error_message = format!("Error while reading image: {}, {}", args.input, e);
            handle_error(error_message.as_str());
            return;
        },
        Ok(img) => { img }
    };

    let clean_image = match ops::clean_and_optimize(&image) {
        Some(img) => img,
        None => image
    };

    if output_is_file && !args.force {
        handle_error("Output already exists. Aborting. Specify --force flag if you want to replace the output");
    }

    match clean_image.save(&args.output) {
        Err(e) => {
            handle_error(e.to_string().as_str());
            return;
        },
        Ok(_) => {
            info!("Successfully exported to: {}", args.output);
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
        let empty = padding::Padding::new(
            ops::count_top_empty_rows(&diamond_image),
            ops::count_bottom_empty_rows(&diamond_image),
            ops::count_left_empty_columns(&diamond_image),
            ops::count_right_empty_columns(&diamond_image)
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
        let clean_image = match ops::clean_and_optimize(&diamond_image) {
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
        let clean_image = match ops::clean_and_optimize(&diamond_image) {
            Some(img) => img,
            None => diamond_image
        };

        assert_eq!(clean_image.width(), 248);
        assert_eq!(clean_image.height(), 208);
    }
}
