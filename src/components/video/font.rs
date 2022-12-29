use image::{DynamicImage, Rgba, io::Reader as ImageReader, GenericImageView};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct CharBitMap {
    pub codepoint: u8,
    pub w: usize,
    pub h: usize,
    pub data: Vec<Rgba<u8>>
}

impl CharBitMap {
    pub fn new(codepoint: u8, w: usize, h: usize, data: Vec<Rgba<u8>>) -> Self {
        Self {
            codepoint,
            w,
            h,
            data
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    // Represents a simple bitmap MONOSPACED font
    pub set_cols: usize, //< in characters
    pub set_rows: usize, //< in characters
    pub set_w: usize,
    pub set_h: usize,
    pub char_w: usize,//< in pixels
    pub char_h: usize,//< in
    pub data: Vec<CharBitMap>,
    pub atlas: DynamicImage
}

impl Font {
    pub fn new(char_w: usize, char_h: usize, file: &Path) -> Self {
        
        let font_atlas = ImageReader::open(file)
                        .unwrap().decode().unwrap();
        let w = font_atlas.width() as usize;
        let h = font_atlas.height() as usize;



        let mut font = Self {
            set_cols: w / char_w,
            set_rows: h / char_h,
            set_w: w,
            set_h: h,
            char_w,
            char_h,
            data: Vec::new(),
            atlas: font_atlas,
        };

        for ch in 0..font.set_cols*font.set_rows {
            font.data.push(get_char(ch as u8 as char, &font))
        }

        font
    }
}

fn get_char(ch: char, font: &Font) -> CharBitMap {
    
    let char_index = ch as usize;

    let char_pos_x = char_index % font.set_cols;
    let char_pos_y = (char_index - char_pos_x) / font.set_cols;

    let char_start_x = char_pos_x * font.char_w;
    let char_start_y = char_pos_y * font.char_h;

    let char_end_x = char_start_x + font.char_w;
    let char_end_y = char_start_y + font.char_h;

    let mut char_data = Vec::new();
    for py in char_start_y..char_end_y{
        for px in char_start_x..char_end_x{
            char_data.push(font.atlas.get_pixel(px as u32, py as u32));
        }
    }

    CharBitMap::new(ch as u8, font.char_w, font.char_h, char_data)

}
