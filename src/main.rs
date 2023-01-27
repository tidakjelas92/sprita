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


fn handle_error(error: &str) {
    error!("{}", error);
    std::process::exit(1);
}

fn main() {
    let args = Arguments::parse();
    initialize_logger();

    let output_is_file = Path::new(&args.output).is_file();

    let image: DynamicImage = match ops::try_read_image(&args.input) {
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
