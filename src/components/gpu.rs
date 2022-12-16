/* 
use arch::*;

type Pixel = Byte;
type Ascii = Byte;

enum Mode {
    Text,
    Graphic,
    RText
}

enum Command {
    Nop,
    Clear,
    Set(Mode),

    SetChar,
    
    SetPixel,
    Blit,

    SetFont
}

const PXSIZE: usize = 0;
const CHSIZE: usize = 0;

struct Gpu {
    mode: Mode,
    pixels: Box<[Pixel; PXSIZE]>,
    asciib: Box<[Ascii; CHSIZE]>,
    asciir: Box<[Ascii; CHSIZE * 2]>,

    line: [Ascii; 80],
    scroll: bool,

    iif: InterruptInterface
} */