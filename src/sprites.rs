// Sprite that contains bit-encoded data (either 1BPP or 2BPP) and corresponding width/height
pub struct Sprite {
    pub bytes: &'static [u8],
    pub width: u32,
    pub height: u32,
    pub flags: u32,
}

#[macro_export]
macro_rules! sprite {
    ($file:expr $(,)?) => {{
        let data: &'static [u8] = include_bytes!($file);
        let width = data[0] as u32;
        let height = data[1] as u32;
        let flags = data[2] as u32;
        Sprite {
            bytes: &data[3..],
            width,
            height,
            flags,
        }
    }};
}
