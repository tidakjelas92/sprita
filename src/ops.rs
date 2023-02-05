use crate::padding::Padding;
use image::{DynamicImage, GenericImageView, ImageError, imageops::FilterType, io::Reader, RgbaImage, Rgba};
use std::cmp::max;

pub fn clean_image(image: DynamicImage) -> DynamicImage {
    let empty = count_empty_padding(&image);

    if is_image_good(&image, &empty) {
        return image;
    }

    if is_padding_zero(&empty) {
        return image;
    }

    clean(image, &empty)
}

pub fn resize_image(image: DynamicImage, max_size: u32) -> DynamicImage {
    let ratio: f32 = image.width() as f32 / image.height() as f32;
    let new_size = max_size;

    let new_width: u32;
    let new_height: u32;

    if image.width() > image.height() {
        new_width = new_size as u32;
        new_height = max((new_size as f32 / ratio).ceil() as u32, 2);
    } else {
        new_width = max((new_size as f32 / ratio).ceil() as u32, 2);
        new_height = new_size as u32;
    }

    image.resize(new_width, new_height, FilterType::Lanczos3)
}

pub fn try_read_image(path: &str) -> Result<DynamicImage, ImageError> {
    Reader::open(path)?
        .with_guessed_format()?
        .decode()
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

fn clean(image: DynamicImage, empty: &Padding) -> DynamicImage {
    let target_width = image.width() - empty.left - empty.right;
    let target_height = image.height() - empty.top - empty.bottom;

    image.crop_imm(empty.left, empty.top, target_width, target_height)
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

pub fn count_empty_padding(image: &DynamicImage) -> Padding {
    Padding::new(
        count_top_empty_rows(image),
        count_bottom_empty_rows(image),
        count_left_empty_columns(image),
        count_right_empty_columns(image)
    )
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

// checks if padding is optimal.
pub fn is_image_good(image: &DynamicImage, empty: &Padding) -> bool {
    let is_dimension_optimized = image.width() % 4 == 0 && image.height() % 4 == 0;
    let empty_is_in_range = (1..4).contains(&empty.left) &&
        (1..4).contains(&empty.right) &&
        (1..4).contains(&empty.top) &&
        (1..4).contains(&empty.bottom);

    is_dimension_optimized && empty_is_in_range
}

pub fn is_padding_zero(padding: &Padding) -> bool {
    padding.left == 0 && padding.right == 0 && padding.top == 0 && padding.bottom == 0
}

pub fn optimize_image(image: DynamicImage) -> DynamicImage {
    let mut padding = Padding::new(0, 0, 0, 0);
    let empty = count_empty_padding(&image);

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

    add_padding(&image, &padding)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn check_test_image() {
        let test_path = "./test.png";
        let test_image = match ops::try_read_image(&test_path) {
            Err(e) => {
                panic!("test image is unreadable, error: {}", e);
            },
            Ok(img) => {
                img
            }
        };
        let empty = padding::Padding::new(
            ops::count_top_empty_rows(&test_image),
            ops::count_bottom_empty_rows(&test_image),
            ops::count_left_empty_columns(&test_image),
            ops::count_right_empty_columns(&test_image)
        );

        assert_eq!(empty.top, 0);
        assert_eq!(empty.left, 0);
        assert_eq!(empty.right, 0);
        assert_eq!(empty.bottom, 0);
    }

    #[test]
    fn clean_test_image() {
        let test_path = "./test.png";
        let test_image = match ops::try_read_image(&test_path) {
            Err(e) => {
                panic!("test image is unreadable, error: {}", e);
            },
            Ok(img) => { img }
        };

        let test_image = ops::clean_image(test_image);
        let empty = ops::count_empty_padding(&test_image);

        assert_eq!(empty.top, 0);
        assert_eq!(empty.left, 0);
        assert_eq!(empty.right, 0);
        assert_eq!(empty.bottom, 0);
    }

    #[test]
    fn optimize_test_image() {
        let test_path = "./test.png";
        let test_image = match ops::try_read_image(&test_path) {
            Err(e) => {
                panic!("test image is unreadable, error: {}", e);
            },
            Ok(img) => { img }
        };

        let test_image = ops::optimize_image(test_image);
        let empty = ops::count_empty_padding(&test_image);

        assert_eq!(empty.top, 1);
        assert_eq!(empty.left, 1);
        assert_eq!(empty.right, 2);
        assert_eq!(empty.bottom, 2);
    }

    #[test]
    fn clean_pig_image() {
        let pig_path = "./pig.png";
        let pig_image = match ops::try_read_image(&pig_path) {
            Err(e) => { panic!("test image is unreadable, error: {}", e); },
            Ok(img) => { img }
        };

        let empty = ops::count_empty_padding(&pig_image);
        assert_eq!(ops::is_image_good(&pig_image, &empty), false);
        assert_eq!(ops::is_padding_zero(&empty), false);

        let pig_image = ops::clean_image(pig_image);
        let empty = ops::count_empty_padding(&pig_image);

        assert_eq!(ops::is_padding_zero(&empty), true);
    }

    #[test]
    fn clean_and_optimize_pig_image() {
        let pig_path = "./pig.png";
        let pig_image = match ops::try_read_image(&pig_path) {
            Err(e) => { panic!("test image is unreadable, error: {}", e); },
            Ok(img) => { img }
        };

        let pig_image = ops::clean_image(pig_image);
        let pig_image = ops::optimize_image(pig_image);
        let empty = ops::count_empty_padding(&pig_image);

        assert_eq!(ops::is_image_good(&pig_image, &empty), true);
    }

    #[test]
    fn resize_pig_image() {
        let pig_path = "./pig.png";
        let pig_image = match ops::try_read_image(&pig_path) {
            Err(e) => { panic!("test image is unreadable, error: {}", e); },
            Ok(img) => { img }
        };

        let pig_image = ops::clean_image(pig_image);
        let empty = ops::count_empty_padding(&pig_image);
        assert_eq!(ops::is_padding_zero(&empty), true);

        let pig_image = ops::resize_image(pig_image, 238);
        assert_eq!(pig_image.width(), 238);
        assert_eq!(pig_image.height(), 201);

        let empty = ops::count_empty_padding(&pig_image);
        assert_eq!(ops::is_padding_zero(&empty), true);

        let pig_image = ops::optimize_image(pig_image);
        let empty = ops::count_empty_padding(&pig_image);
        assert_eq!(ops::is_image_good(&pig_image, &empty), true);
        assert_eq!(pig_image.width(), 240);
        assert_eq!(pig_image.height(), 204);
    }
}
