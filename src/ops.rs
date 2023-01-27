use crate::padding::Padding;
use image::{DynamicImage, GenericImageView, ImageError, io::Reader, RgbaImage, Rgba};

pub fn clean_and_optimize(image: &DynamicImage) -> Option<DynamicImage> {
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

fn clean(image: &DynamicImage, empty: &Padding) -> Option<DynamicImage> {
    if empty.left == 0 && empty.right == 0 && empty.top == 0 && empty.bottom == 0 {
        return None;
    }

    let target_width = image.width() - empty.left - empty.right;
    let target_height = image.height() - empty.top - empty.bottom;

    Some(image.crop_imm(empty.left, empty.top, target_width, target_height))
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

fn is_image_good(image: &DynamicImage, empty: &Padding) -> bool {
    let is_dimension_optimized = image.width() % 4 == 0 && image.height() % 4 == 0;
    let empty_is_in_range = (1..4).contains(&empty.left) &&
        (1..4).contains(&empty.right) &&
        (1..4).contains(&empty.top) &&
        (1..4).contains(&empty.bottom);

    is_dimension_optimized && empty_is_in_range
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn count_empty_clean_image() {
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
    fn clean_and_optimize_clean_image() {
        let test_path = "./test.png";
        let test_image = match ops::try_read_image(&test_path) {
            Err(e) => {
                panic!("diamond image is unreadable, error: {}", e);
            },
            Ok(img) => {
                img
            }
        };
        let clean_image = match ops::clean_and_optimize(&test_image) {
            Some(img) => img,
            None => test_image
        };

        assert_eq!(clean_image.width(), 8);
        assert_eq!(clean_image.height(), 8);
    }

    #[test]
    fn clean_and_optimzie_unclean_image() {
        let test_path = "./test_unclean.png";
        let test_image = match ops::try_read_image(&test_path) {
            Err(e) => {
                panic!("diamond image is unreadable, error: {}", e);
            },
            Ok(img) => {
                img
            }
        };
        let clean_image = match ops::clean_and_optimize(&test_image) {
            Some(img) => img,
            None => test_image
        };

        assert_eq!(clean_image.width(), 8);
        assert_eq!(clean_image.height(), 8);
    }
}
