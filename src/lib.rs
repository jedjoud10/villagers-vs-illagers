#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
use wasm4::*;

static mut GAME: Option<Game> = None;

// p1: villager
// p2: illager
struct Game {
    villager: u8,
    illager: u8,
    cursors: [u8; 2],
    old_gamepad: [u8; 2],
    new_gamepad: [u8; 2],
    grid: [u8; 24],
}

impl Game {
    unsafe fn new() -> Self {
        *PALETTE = [
            0xd7bbad, 0xbf8a6e, 0x57795b, 0x26180e
        ];

        Self {
            villager: 5, 
            illager: 5,
            cursors: [0; 2],
            new_gamepad: [0; 2],
            old_gamepad: [*GAMEPAD1, *GAMEPAD2],
            grid: [0; 24],
        }
    }
    
    unsafe fn run(&mut self) {
        self.fetch_input();
        self.move_cursor();
        self.draw_background_grid();
        self.draw_footer();
        self.draw_cursors();
        
        //debug_palette();
    }

    // Fetch gamepad input. Also works in multiplayer. Only supports 2 players
    unsafe fn fetch_input(&mut self) {
        const GAMEPADS: [*const u8; 2] = [GAMEPAD1, GAMEPAD2];
        for index in 0..2 {
            let last = self.old_gamepad[index];
            let current = *GAMEPADS[index];
            let new = current & (last ^ current);
            self.old_gamepad[index] = current;
            self.new_gamepad[index] = new;
        }
    }

    // Move the cursor based on the input
    unsafe fn move_cursor(&mut self) {
        for index in 0..2 {
            let x = self.new_gamepad[index];
            
            // TODO: Refactor
            if x & BUTTON_UP != 0 {
                self.cursors[index] = self.cursors[index].saturating_sub(16);
            } else if x & BUTTON_DOWN != 0 {
                self.cursors[index] = self.cursors[index].saturating_add(16);
            } else if x & BUTTON_LEFT != 0 {
                self.cursors[index] = self.cursors[index].saturating_sub(1);
            } else if x & BUTTON_RIGHT != 0 {
                self.cursors[index] = self.cursors[index].saturating_add(1);
            }

            self.cursors[index] = self.cursors[index].clamp(0, 191);
        }
    }

    // Draw a footer containing points, classes to summon, and current selected cell
    unsafe fn draw_footer(&mut self) {
        *DRAW_COLORS = 0b1000000;
        rect(0, 120, 160, 40);
    
        *DRAW_COLORS = 0b0100_0000_0000_0100;
        text("V: ", 2, 122);
        text("I: ", 2, 132);
        text([self.villager + 48], 16, 122);
        text([self.illager + 48], 16, 132);
    }

    // Draw the background tiles for the grid
    unsafe fn draw_background_grid(&self) {
        for x in 0..16i32 {
            for y in 0..12i32 {
                *DRAW_COLORS = if ((x % 2) == 0) ^ ((y % 2) == 0) { 2 } else { 3 };
                rect(x * 10, y * 10, 10, 10)
            }
        }
    }

    // Draw the player cursors. Different colors assigned to each team
    unsafe fn draw_cursors(&self) {
        for (index, pos) in self.cursors.iter().enumerate() {
            let posy = pos / 16;
            let posx = pos % 16;

            const COLORS: [u8; 2] = [0b1000000, 0b0010000];
            *DRAW_COLORS = COLORS[index] as u16;
            rect((posx * 10) as i32,(posy * 10) as i32, 10, 10);
        }
    }
    
    // Draw a debug palette at the bottom right corner
    unsafe fn debug_palette(&self) {
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