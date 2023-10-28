#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use wasm4::*;

static mut GAME: Option<Game> = None;

struct Game {
    villager: u8,
    illager: u8,
}

impl Game {
    unsafe fn new() -> Self {
        *PALETTE = [
            0xd7bbad, 0xbf8a6e, 0x57795b, 0x26180e
        ];

        Self {
            villager: 5, 
            illager: 5,
        }
    }
    
    unsafe fn run(&mut self) {
        for x in 0..16i32 {
            for y in 0..12i32 {
                *DRAW_COLORS = if ((x % 2) == 0) ^ ((y % 2) == 0) { 2 } else { 3 };
                rect(x * 10, y * 10, 10, 10)
            }
        }
    
        *DRAW_COLORS = 0b1000000;
        rect(0, 120, 160, 40);
    
        *DRAW_COLORS = 0b0100_0000_0000_0100;
        text("V: ", 2, 122);
        text("I: ", 2, 132);
        text([self.villager + 48], 16, 122);
        text([self.illager + 48], 16, 132);

        *DRAW_COLORS = 0b0100_0000_0000_0001;
        rect(150, 120, 10, 10);
        *DRAW_COLORS = 0b0100_0000_0000_0010;
        rect(150, 130, 10, 10);
        *DRAW_COLORS = 0b0100_0000_0000_0011;
        rect(150, 140, 10, 10);
        *DRAW_COLORS = 0b0100_0000_0000_0100;
        rect(150, 150, 10, 10);
    }
}

#[no_mangle]
unsafe fn start() {
    GAME = Some(Game::new());
}

#[no_mangle]
unsafe fn update() {
    GAME.as_mut().unwrap().run();
}