#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;
mod sprites;
pub use sprites::*;
use wasm4::*;

static mut GAME: Option<Game> = None;

// Price constants
const VINDICATOR: u8 = 1;
const VILLAGER: u8 = VINDICATOR;
const PILLAGER: u8 = 2;
const FARMER: u8 = PILLAGER;
const EVOKER: u8 = 3;
const SMITH: u8 = EVOKER;

// Village stuff goes first since P1 is controlling the villagers
const PRICES: [u8; 6] = [
    VILLAGER, FARMER, SMITH,
    VINDICATOR, PILLAGER, EVOKER,
];

// Entities associated with illagers (vex included)
#[derive(Clone, Copy)]
enum IllagerClan {
    Vindicator,
    Pillager,
    Evoker,
    Vex,
}

// Unique state for every type of illager
#[derive(Clone, Copy)]
enum IllagerState {
    Idle,
    Action,
}

// Unique state for golem
#[derive(Clone, Copy)]
enum GolemState {
    Attack,
    Broken,
    Idle
}

// Entities associated with villagers (golems included)
#[derive(Clone, Copy)]
enum VillagerClan {
    Villager,
    Farmer,
    Smith,
    Golem(GolemState),
}

// Potential state of each cell
#[derive(Clone, Copy)]
enum CellState {
    Empty,

    // Illager type and corresponding state
    IllagerClan(IllagerClan, IllagerState),

    // Villagers have no different state types
    VillagerClan(VillagerClan),

    // Index repsenting a number from 0-4.
    // 0: bottom left
    // 1: bottom right
    // 2: top left
    // 3: top right
    House(u8)

    /* In case we want to have more map variety (given that things will be randomly generated)
    Tree1,
    Tree2,
    Tree3,
    Tree4,
    Pen1,
    Pen2,
    Pen3,
    Pen4,
    Church1,
    Church2,
    Church3,
    Church4
    */
}


/*
TODO:
Sprites
- Villager
- Farmer
- Smith
- Iron golem
- Vindicator
- Pillager
- Evoker
- Vex
*/

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
            villager: 9,
            illager: 9,
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
            let grid_pos = self.cursors[index];
            self.cursors[index] = if x & BUTTON_UP != 0 {
                grid_pos.saturating_sub(16)
            } else if x & BUTTON_DOWN != 0 {
                grid_pos.saturating_add(16)
            } else if x & BUTTON_LEFT != 0 {
                grid_pos.saturating_sub(1)
            } else if x & BUTTON_RIGHT != 0 {
                grid_pos.saturating_add(1)
            } else {
                grid_pos
            }   
            .clamp(0, 191);

    
            let selected = &mut self.current_selected_class[index];

            // Cycle current selected class
            if new & BUTTON_2 != 0 {
                *selected += 1;
                *selected = *selected % 3;
            }

            // Place currently selected class
            if new & BUTTON_1 != 0 {
                let points: &mut u8 = if index == 0 { &mut self.villager } else { &mut self.illager };

                // make sure the cell is empty so we can place our shit there
                if matches!(self.grid[grid_pos as usize], CellState::Empty) {
                    // checked sub to make sure we don't cause a crash (also saves us from
                    // manually comparing to check if we have enough points to spend) 
                    if let Some(new_points) = points.checked_sub(PRICES[*selected as usize + 3 * index]) {
                        *points = new_points;
    
                        // logic that handles setting new classes
                        // basically overwrite the cell state, so we should have a check (even before subtracting the points) to make
                        // sure that the position is valid
                        let cell = &mut self.grid[grid_pos as usize];

                        // "index" is play index (where 0 is villager and 1 is illager)
                        // "selected" is the selected class index (0..3)
                        *cell = match (index, selected) {
                            // villager clan classes
                            (0, 0) => CellState::VillagerClan(VillagerClan::Villager),
                            (0, 1) => CellState::VillagerClan(VillagerClan::Farmer),
                            (0, 2) => CellState::VillagerClan(VillagerClan::Smith),
    
                            // illager clan classes
                            (1, 0) => CellState::IllagerClan(IllagerClan::Pillager, IllagerState::Idle),
                            (1, 1) => CellState::IllagerClan(IllagerClan::Vindicator, IllagerState::Idle),
                            (1, 2) => CellState::IllagerClan(IllagerClan::Evoker, IllagerState::Idle),
    
                            _ => unreachable!()
                        };
                    }
                }
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
        
        let mut buffer = itoa::Buffer::new();
        text(buffer.format(self.villager), 16, 122);
        text(buffer.format(self.illager), 16, 132);

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
        *DRAW_COLORS = 0b0100_0011_0010_0001;

        for (index, state) in self.grid.iter().enumerate() {
            let dst_grid_x = index % 16;
            let dst_grid_y = index / 16;
            let dst_x = (dst_grid_x * 10) as i32;
            let dst_y = (dst_grid_y * 10) as i32;

            // blit_sub(&SPRITE, dst_x, dst_y, 10, 10, 0, 0, 80, SPRITE_FLAGS);

            match state {
                CellState::Empty => {},

                CellState::IllagerClan(_type, state) => {
                    // src x pos inside the sprite sheet that we will blit from
                    let src_x = match _type {
                        IllagerClan::Vindicator => 0,
                        IllagerClan::Pillager => 10,
                        IllagerClan::Evoker => 20,
                        IllagerClan::Vex => 30,
                    };

                    // src y pos from the sprite sheet
                    let src_y = match state {
                        IllagerState::Idle => 0,
                        IllagerState::Action => 10,
                    };

                    blit_sub(&SPRITE, dst_x, dst_y, 10, 10, src_x, src_y, 80, SPRITE_FLAGS);
                },

                CellState::VillagerClan(_type) => {
                    // src x and src y positions inside the sprite sheet
                    let (src_x, src_y) = match _type {
                        VillagerClan::Villager => (40, 0),
                        VillagerClan::Farmer => (50, 0),
                        VillagerClan::Smith => (60, 0),
                        VillagerClan::Golem(golem) => match golem {
                            GolemState::Attack => (60, 10),
                            GolemState::Broken => (70, 10),
                            GolemState::Idle => (70, 0),
                        },
                    };

                    blit_sub(&SPRITE, dst_x, dst_y, 10, 10, src_x, src_y, 80, SPRITE_FLAGS);
                },
                
                CellState::House(_) => todo!(),
            };
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
