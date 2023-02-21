use crate::{arguments::Arguments, error::handle_error, image_ops};
use image::{DynamicImage, ImageError};
use std::{cmp::max, fs::{canonicalize, create_dir_all, ReadDir, read_dir}, path::{Path, PathBuf}, thread, thread::JoinHandle};

pub fn process_args(args: &Arguments) {
    let input_path = Path::new(&args.input);
    if !input_path.exists() {
        handle_error(format!("input: {} does not exist!", &args.input).as_str());
    }

    if args.downsize {
        if args.max_size.is_none() {
            handle_error("downsizing operation requires a --max-size parameter to be specified.");
        }

        let max_size = args.max_size.unwrap();

        if max_size < 2 {
            handle_error("--max-size parameter is too small.");
        }

        if max_size % 4 != 0 {
            warn!("--max-size parameter is not multiple of 4, it will be rounded up to the closest multiple of 4.");
        }
    }

    let output_path = Path::new(&args.output);
    match output_path.parent() {
        None => { handle_error(format!("output: {} is an invalid path.", &args.output).as_str()); },
        Some(parent) => {
            // passing only the file name will cause parent() to return Some("")
            if parent.to_path_buf() != PathBuf::from("") {
                create_dir(parent);
            }
        }
    }

    if input_path.is_file() {
        if output_path.is_file() && !args.force {
            handle_error("Output path already contains a file. Specify --force if the program needs to overwrite it.");
        }

        if output_path.is_dir() {
            handle_error("Output path is a directory.");
        }

        match process_file_path(&args.input, &args.output, args.downsize, args.max_size) {
            Err(e) => { handle_error(format!("input: {}\n    output: {}\n    error: {}", &args.input, &args.output, e.to_string()).as_str()); },
            Ok(_) => { }
        }
        return;
    }

    create_dir(&output_path);

    match read_dir(input_path) {
        Ok(paths) => { process_dir_path(paths, &output_path, args.downsize, args.max_size, args.force); },
        Err(e) => { handle_error(&format!("input: {}. {}", &args.input, &e.to_string())); }
    }
}

fn create_dir(path: &Path) {
    if path.is_dir() { return; }

    info!("{} is not a dir. Creating one...", path.to_str().unwrap());
    match create_dir_all(&path) {
        Ok(_) => { info!("Successfully created dir at: {}", path.to_str().unwrap()); },
        Err(e) => { handle_error(&e.to_string()); }
    }
}

fn process_dir_path(paths_read_dir: ReadDir, output_dir_path: &Path, downsize: bool, max_size: Option<u32>, force: bool) {
    let mut handles = Vec::<JoinHandle<()>>::new();
    let paths = get_entries(paths_read_dir, output_dir_path, force);

    for path in paths {
        let output_path = output_dir_path.join(path.file_name().unwrap_or(&path.clone().into_os_string()));

        let handle = thread::spawn(move || {
            match process_file_path(
                &path.into_os_string().into_string().unwrap(),
                &output_path.into_os_string().into_string().unwrap(),
                downsize,
                max_size
            ) {
                Err(e) => {
                    // ImageError::IoError occurs when input is a dir.
                    // ImageError::Unsupported occurs when the input is not an image.
                    // these errors are ignored, but the rest are printed.
                    match e {
                        ImageError::IoError(_) => { },
                        ImageError::Unsupported(_) => { },
                        _ => { error!("error: {}. Skipping...", e.to_string()); }
                    }
                },
                Ok(_) => { }
            }
        });

        handles.push(handle);
    }

    for handle in handles.into_iter() {
        handle.join().unwrap();
    }
}

fn get_entries(dir: ReadDir, output_dir_path: &Path, force: bool) -> Vec<PathBuf> {
    let mut paths = Vec::<PathBuf>::new();

    for path in dir {
        match &path {
            Err(e) => {
                error!("{}. path: {:?} Skipping...", &e.to_string(), path);
                continue;
            },
            Ok(entry) => {
                let output_path_buf = output_dir_path.join(entry.file_name());

                let output_path = output_path_buf.as_path();
                if output_path_buf.as_path().is_file() && !force {
                    error!("Output path: {} already contains a file. Specify --force if the program needs to overwrite it. Skipping...", canonicalize(output_path).unwrap().display());
                    continue;
                }

                paths.push(entry.path());
            }
        }
    }

    paths
}

fn process_file_path(input: &String, output: &String, downsize: bool, max_size: Option<u32>) -> Result<(), ImageError> {
    let output_path = Path::new(output);

    let mut image: DynamicImage = image_ops::try_read_image(input)?;
    image = process_image(image, downsize, max_size);
    image.save(output)?;

    info!("Successfully exported to: {}", canonicalize(output_path).unwrap().display());
    Ok(())
}

fn process_image(mut image: DynamicImage, downsize: bool, max_size: Option<u32>) -> DynamicImage {
    image = image_ops::clean_image(image);

    if downsize {
        // this is to give room for padding.
        image = downsize_image(image, max_size.unwrap() - 2);
    }

    image = image_ops::optimize_image(image);

    image
}

fn downsize_image(mut image: DynamicImage, max_size: u32) -> DynamicImage {
    let current_max_size = max(image.width(), image.height());
    if current_max_size <= max_size { return image; }

    image = image_ops::resize_image(image, max_size);

    image
}
