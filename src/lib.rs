#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
mod sprites;
pub use sprites::*;
use wasm4::*;

static mut GAME: Option<Game> = None;

// Potential state of each cell
#[derive(Clone, Copy)]
enum CellState {
    Empty,
    // add villager states
    // add illager states
    // add house state   
}


// p1: villager
// p2: illager
struct Game {
    villager: u8,
    illager: u8,
    cursors: [u8; 2],
    old_gamepad: [u8; 2],
    new_gamepad: [u8; 2],
    current_selected_class: [u8; 2],
    grid: [CellState; 192],
}

impl Game {
    unsafe fn new() -> Self {
        // grey, beige, green, brown
        *PALETTE = [0xd7bbad, 0xbf8a6e, 0x57795b, 0x26180e];

        Self {
            villager: 5,
            illager: 5,
            cursors: [0; 2],
            new_gamepad: [0; 2],
            old_gamepad: [*GAMEPAD1, *GAMEPAD2],
            current_selected_class: [0, 0],
            grid: [CellState::Empty; 192],
        }
    }

    unsafe fn run(&mut self) {
        self.fetch_input();
        self.draw_background();
        self.draw_sprites();
        self.draw_footer();
        self.draw_cursors();
        self.debug_palette();
    }

    // Fetch gamepad input. Also works in multiplayer. Only supports 2 players
    // Also moves the appropriate selectors if necessary
    unsafe fn fetch_input(&mut self) {
        const GAMEPADS: [*const u8; 2] = [GAMEPAD1, GAMEPAD2];
        for index in 0..2 {
            let last = self.old_gamepad[index];
            let current = *GAMEPADS[index];
            let new = current & (last ^ current);
            self.old_gamepad[index] = current;
            self.new_gamepad[index] = new;

            // Move cursor on grid
            let x = self.new_gamepad[index];
            let pos = self.cursors[index];
            self.cursors[index] = if x & BUTTON_UP != 0 {
                pos.saturating_sub(16)
            } else if x & BUTTON_DOWN != 0 {
                pos.saturating_add(16)
            } else if x & BUTTON_LEFT != 0 {
                pos.saturating_sub(1)
            } else if x & BUTTON_RIGHT != 0 {
                pos.saturating_add(1)
            } else {
                pos
            }   
            .clamp(0, 191);

            // Cycle current selected class
            if new & BUTTON_2 != 0 {
                let selected = &mut self.current_selected_class[index];
                *selected += 1;
                *selected = *selected % 3;
            }
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

        *DRAW_COLORS = 0b0100_0011_0010_0001;

        // Draw mini-icon for selectors
        for index in 0..2i32 {
            /*
            match index {
                0 => {
                    //blit(&SPRITE, 120, 122, SPRITE_WIDTH, SPRITE_HEIGHT, SPRITE_FLAGS);

                },
                1 => {

                }
                _ => {}
            }
            */
        }
    }

    // Draw the background color
    unsafe fn draw_background(&self) {
        *DRAW_COLORS = 0b0011_0011_0011_0011;
        rect(0, 0, 160, 120);
    }

    // Draw the grid with the appropriate sprites
    unsafe fn draw_sprites(&self) {
        for (index, state) in self.grid.iter().enumerate() {
            // TODO: Draw appropriate sprite here
            match state {
                CellState::Empty => {},
            }
        }
    }

    // Draw the player cursors. Different colors assigned to each team
    unsafe fn draw_cursors(&self) {
        for (index, selector_position) in self.cursors.iter().enumerate() {
            let posy = selector_position / 16;
            let posx = selector_position % 16;

            const COLORS: [u8; 2] = [0b1000000, 0b0010000];
            *DRAW_COLORS = COLORS[index] as u16;
            rect((posx * 10) as i32, (posy * 10) as i32, 10, 10);
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
