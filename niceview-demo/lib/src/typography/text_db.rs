/// The width of each glyph.
pub const CHAR_WIDTH: usize = 5;
/// The height of each glyph.
pub const CHAR_HEIGHT: usize = 7;

/// The char is the ascii value of the character - 0x20 (start of printable characters).
/// All chars take up an area of 5x7 (5 pixels wide and 7 pixels tall).
const CHARS: [[u8; CHAR_HEIGHT]; 59] = [
    // 0x20: Space ' '
    [0, 0, 0, 0, 0, 0, 0],
    // 0x21: Exclaimation mark '!'
    [0, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100],
    // 0x22: Double quotes '"'
    [0b01010, 0b01010, 0, 0, 0, 0, 0],
    // 0x23: Number sign '#'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x24: Dollar '$'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x25: Per cent sign '%'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x26: Ampersand '&'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x27: Single quote '''
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x28: Open parenthesis '('
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x29: Close parenthesis ')'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x2A: Asterisk '*'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x2B: Plus '+'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x2C: Comma ','
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x2D: Hyphen-minus '-'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x2E: Period '.'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x2F: Slash '/'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x30: Zero '0'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x31: One '1'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x32: Two '2'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x33: Three '3'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x34: Four '4'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x35: Five '5'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x36: Six '6'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x37: Seven '7'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x38: Eight '8'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x39: Nine '9'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x3A: Colon ':'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x3B: Semicolon ';'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x3C: Less than '<'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x3D: Equals '='
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x3E: Greater than '>'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x3F: Question Mark '?'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x40: At sign '@'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x41: Uppercase 'A'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x42: Uppercase 'B'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x43: Uppercase 'C'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x44: Uppercase 'D'
    [
        0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
    ],
    // 0x45: Uppercase 'E'
    [
        0b11111, 0b10000, 0b10000, 0b11111, 0b10000, 0b10000, 0b11111,
    ],
    // 0x46: Uppercase 'F'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x47: Uppercase 'G'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x48: Uppercase 'H'
    [
        0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
    ],
    // 0x49: Uppercase 'I'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x4A: Uppercase 'J'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x4B: Uppercase 'K'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x4C: Uppercase 'L'
    [
        0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
    ],
    // 0x4D: Uppercase 'M'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x4E: Uppercase 'N'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x4F: Uppercase 'O'
    [
        0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
    ],
    // 0x50: Uppercase 'P'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x51: Uppercase 'Q'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x52: Uppercase 'R'
    [
        0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b10001,
    ],
    // 0x53: Uppercase 'S'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x54: Uppercase 'T'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x55: Uppercase 'U'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x56: Uppercase 'V'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x57: Uppercase 'W'
    [
        0b10001, 0b10001, 0b10001, 0b10001, 0b10101, 0b11011, 0b10001,
    ],
    // 0x58: Uppercase 'X'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x59: Uppercase 'Y'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
    // 0x5A: Uppercase 'Z'
    // TODO
    [0, 0, 0, 0, 0, 0, 0],
];

const START_OFFSET: usize = 0x20;

/// Weather a specific pixel is part of the glyph specified by 'char'.
/// Coordinate system is from top-left (0x0) to bottom-right (4x6), glyphs are 5x7 big.
/// Illegal chars or indices out of bounds always return false.
pub fn is_pixel_for_char(char: char, x: usize, y: usize) -> bool {
    let char = char as usize;
    if char < START_OFFSET {
        // Not a printable char.
        return false;
    }

    if x >= 5 || y >= 7 {
        // Out of bounds.
        return false;
    }

    let index = char - START_OFFSET;
    log::info!("CHAR: {char:x} ({x}, {y})");
    let arr = CHARS[index];
    let row = arr[y];
    log::info!("ROW: {row:05b}");
    let col = 4 - x;
    ((row >> col) & 1) == 1
}
