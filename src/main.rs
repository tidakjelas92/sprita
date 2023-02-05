#[macro_use] extern crate log;
extern crate simplelog;

mod padding;
mod ops;

use clap::Parser;
use image::DynamicImage;
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
    force: bool,

    #[arg(short, long)]
    resize: bool,

    #[arg(short, long)]
    max_size: Option<i32>
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

fn handle_error(error: &str) {
    error!("{}", error);
    std::process::exit(1);
}

fn process_image(mut image: DynamicImage, args: &Arguments) -> DynamicImage {
    image = ops::clean_image(image);

    if args.resize {
        // this is to give room for padding.
        let max_size = (args.max_size.unwrap() as u32) - 2;
        image = ops::resize_image(image, max_size);
    }

    image = ops::optimize_image(image);

    image
}

fn main() {
    let args = Arguments::parse();
    initialize_logger();

    let output_is_file = Path::new(&args.output).is_file();

    let mut image: DynamicImage = match ops::try_read_image(&args.input) {
        Err(e) => {
            let error_message = format!("Error while reading image: {}, {}", args.input, e);
            handle_error(error_message.as_str());
            return;
        },
        Ok(img) => { img }
    };

    image = process_image(image, &args);

    if output_is_file && !args.force {
        handle_error("Output already exists. Aborting. Specify --force flag if you want to replace the output");
    }

    match image.save(&args.output) {
        Err(e) => {
            handle_error(e.to_string().as_str());
            return;
        },
        Ok(_) => {
            info!("Successfully exported to: {}", args.output);
        }
    }
}
