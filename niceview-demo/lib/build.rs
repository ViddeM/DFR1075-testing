use eg_font_converter::{FontConverter, Mapping};

const RAW_FONT_DIR: &'static str = "./resources/fonts/bdf";
const OUTPUT_DIR: &'static str = "./src/typography/generated/";

/// Parse, compile and re-store fonts.
fn main() {
    load_font("Tamzen10x20r", "tamzen_10x20r");
    load_font("Tamzen10x20b", "tamzen_10x20b");
    load_font("5x7", "small_5x7");

    println!("cargo:rerun-if-changed=build.rs")
}

fn load_font<'a>(file_name: &'a str, output_name: &'a str) {
    let font = FontConverter::new(format!("{RAW_FONT_DIR}/{file_name}.bdf"), &output_name)
        .glyphs(Mapping::Iso8859_1)
        .missing_glyph_substitute(' ')
        .convert_eg_bdf()
        .expect("Failed to convert large font");

    font.save(OUTPUT_DIR).expect("Failed to save font");
}
