use crate::{arguments::Arguments, error::handle_error, image_ops};
use image::DynamicImage;
use std::path::Path;

pub fn process_args(args: &Arguments) {
    let input_path = Path::new(&args.input);
    let output_path = Path::new(&args.output);

    if input_path.is_file() {
        let output_is_file = output_path.is_file();

        let mut image: DynamicImage = match image_ops::try_read_image(&args.input) {
            Err(e) => {
                let error_message = format!("Error while reading image: {}, {}", &args.input, e);
                handle_error(error_message.as_str());
                return;
            },
            Ok(img) => { img }
        };

        image = process_image(image, args.downsize, args.max_size);

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

        return;
    }
}

fn process_image(mut image: DynamicImage, downsize: bool, max_size: Option<i32>) -> DynamicImage {
    image = image_ops::clean_image(image);

    if downsize {
        // this is to give room for padding.
        let max_size = (max_size.unwrap() as u32) - 2;
        image = image_ops::resize_image(image, max_size);
    }

    image = image_ops::optimize_image(image);

    image
}
