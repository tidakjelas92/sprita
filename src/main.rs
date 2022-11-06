#[macro_use] extern crate log;
extern crate simplelog;

use clap::{command, Arg, ArgMatches};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TerminalMode, TermLogger};
use std::path::Path;

fn parse_arguments() -> ArgMatches {
    command!()
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

fn main() {
    let args = parse_arguments();
    initialize_logger();

    let input_path = args.get_one::<String>("input").unwrap();
    let output_path = args.get_one::<String>("output").unwrap();

    let input_is_file = Path::new(input_path).is_file();
    let output_is_file = Path::new(output_path).is_file();

    if !input_is_file {
        error!("Input path is not a file! Aborting...");
        std::process::exit(1);
    }

    if output_is_file {
        warn!("Output already exists!");
    }
}
