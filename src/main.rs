#[macro_use] extern crate log;
extern crate simplelog;

mod arguments;
mod error;
mod image_ops;
mod logging;
mod ops;
mod padding;

use clap::Parser;

fn main() {
    let args = arguments::Arguments::parse();
    logging::initialize();
    ops::process_args(&args);
}
