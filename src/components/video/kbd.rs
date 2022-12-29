#[derive(Debug)]
pub struct Keyboard {
    // El teclado expone cuatro registros:
    // carácter: 1 byte                 R
    // modificadores: 1 byte            R -> hay que convertir de u32 a u8
    // code: 1 ( a lo mejor 2) byte     R
    // modo: 1 byte (Scancode/Keycode)  W
    pub character: u8,
    pub code: u8,
    pub modifiers: u8,
    pub mode: u8
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Nop,
    SetMode
}

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    ScanCode,
    KeyCode
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            character: 0,
            code: 0,
            modifiers: 0,
            mode: 0
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode as u8;
    }
}