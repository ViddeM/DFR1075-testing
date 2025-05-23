use std::path::Path;

use build_xbm::generate_for_xbm_file;
use eg_font_converter::{FontConverter, Mapping};

const RAW_FONT_DIR: &'static str = "./resources/fonts/bdf";
const FONT_OUTPUT_DIR: &'static str = "./src/typography/generated/";

const RAW_IMAGE_DIR: &'static str = "./resources/bitmap";

mod build_xbm;

/// Parse, compile and re-store fonts.
fn main() {
    load_font("Tamzen10x20r", "tamzen_10x20r");
    load_font("Tamzen10x20b", "tamzen_10x20b");
    load_font("5x7", "small_5x7");

    let image_path = format!("{RAW_IMAGE_DIR}/my_first_bitmap.xbm");
    let image_path = Path::new(&image_path);

    generate_for_xbm_file(image_path, "my_file.rs");

    println!("cargo:rerun-if-changed=build.rs")
}

fn load_font<'a>(file_name: &'a str, output_name: &'a str) {
    let font = FontConverter::new(format!("{RAW_FONT_DIR}/{file_name}.bdf"), &output_name)
        .glyphs(Mapping::Iso8859_1)
        .missing_glyph_substitute(' ')
        .convert_eg_bdf()
        .expect("Failed to convert large font");

    font.save(FONT_OUTPUT_DIR).expect("Failed to save font");
}
