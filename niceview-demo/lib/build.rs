use eg_font_converter::{FontConverter, Mapping};

/// Parse, compile and re-store fonts.
fn main() {
    let font = FontConverter::new("./resources/fonts/bdf/Tamzen10x20r.bdf", "tamzen_10x20r")
        .glyphs(Mapping::Iso8859_1)
        .missing_glyph_substitute(' ')
        .convert_eg_bdf()
        .expect("Failed to convert font");

    font.save("./src/typography/generated/")
        .expect("Failed to save font");

    println!("cargo:rerun-if-changed=build.rs")
}
