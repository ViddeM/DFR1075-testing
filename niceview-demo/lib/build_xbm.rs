use proc_macro2::TokenStream;
use std::{fs, io::Write, path::Path, str::Lines};

use quote::quote;

use my_image::Image;

#[path = "src/image.rs"]
mod my_image;

const C_DEFINE_PREFIX: &'static str = "#define";
const C_STATIC_UNASIGNED_CHAR: &'static str = "static unsigned char";
const IMAGE_OUTPUT_DIR: &'static str = "./src/images/generated/";

pub fn generate_for_xbm_file(file_path: &Path, output_file_name: &'static str) {
    let file_content = fs::read_to_string(file_path).expect("Failed to read xbm file");

    let mut lines = file_content.lines();

    let width = next_starting_with(&mut lines, C_DEFINE_PREFIX)
        .expect("width define")
        .split(" ")
        .last()
        .expect("Expected a define part?")
        .parse::<usize>()
        .expect("Failed to parse image width");
    let height = next_starting_with(&mut lines, C_DEFINE_PREFIX)
        .expect("height define")
        .split(" ")
        .last()
        .expect("Expected a define part?")
        .parse::<usize>()
        .expect("Failed to parse image height");

    // Skip the actual static unsigned char line as it contains no data.
    next_starting_with(&mut lines, C_STATIC_UNASIGNED_CHAR).expect("Unable to find xbm data");

    let remainer = lines.collect::<String>();
    let pixels = remainer
        .strip_suffix("};")
        .expect("Failed to strip suffix of xbm file");

    let image_bytes = pixels
        .replace("\n", "")
        .replace(" ", "")
        .split(",")
        .map(|h| h.trim_start_matches("0x"))
        .map(|hex_bits| u8::from_str_radix(hex_bits, 16).expect("Failed to parse hex to u16"))
        .collect::<Vec<u8>>();

    let byte_width = convert_image_width_to_byte_width(width);

    let rows: TokenStream = image_bytes
        .chunks_exact(byte_width)
        .map(|s| {
            let a = s.iter().map(|u| quote! { #u, }).collect::<TokenStream>();
            quote! { [ #a ], }
        })
        .collect();

    let output = quote! {
        use crate::image::Image;

        pub const MY_COOL_IMAGE: Image<#width, #byte_width, #height>  = Image {
            rows: [ #rows ]
        };
    }
    .to_string();

    let output_file_path = format!("{IMAGE_OUTPUT_DIR}/{output_file_name}");
    let mut file =
        fs::File::create(output_file_path).expect("Failed to create image rust file output");

    file.write_all(output.as_bytes())
        .expect("Failed to write rust file for image");
}

fn convert_image_width_to_byte_width(image_width: usize) -> usize {
    (image_width + (8 - (image_width % 8))) / 8
}

fn next_starting_with<'a>(lines: &'a mut Lines<'_>, prefix: &'static str) -> Result<&'a str, ()> {
    while let Some(l) = lines.next() {
        if l.starts_with(prefix) {
            return Ok(l);
        }
    }

    Err(())
}
