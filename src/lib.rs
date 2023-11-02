#[cfg(feature = "buddy-alloc")]
mod alloc;
mod sprites;
mod terrain;
mod wasm4;
mod slotmap;
use std::{collections::HashMap, num::NonZeroU8, u8, mem::transmute};

use nonmax::NonMaxU8;
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
const PRICES: [u8; 6] = [VILLAGER, FARMER, SMITH, VINDICATOR, PILLAGER, EVOKER];

// Entities associated with illagers (vex included)
#[derive(Clone, Copy)]
pub enum IllagerClan {
    Vindicator,
    Pillager,
    Evoker { vex_ids: [Option<NonMaxU8>; 2] },
    Vex { id: NonMaxU8 },
}

// Unique state for every type of illager
#[derive(Clone, Copy)]
pub enum IllagerState {
    Idle,
    Action,
}

// Unique state for golem
#[derive(Clone, Copy)]
pub enum GolemState {
    Attack,
    Broken,
    Idle,
}

// Stored in the game world. Describes a link between a minion (vec / golem) and a parent (evoker / smith)
// Keeps track of the minion position, and spawner position
// It is the responsability of the spawner to update its pos values whenever it moves
// It is the responsibility of the minion to update its pos values whenever it moves
struct MinionLink {
    minion_id: NonMaxU8,
    minion_position_index: u8,
    parent_position_index: u8,
}

// Entities associated with villagers (golems included)
#[derive(Clone, Copy)]
pub enum VillagerClan {
    Villager,
    Farmer,
    Smith { golem_id: Option<NonMaxU8> },
    Golem { id: NonMaxU8, state: GolemState },
}

// Currents state for the buildings
#[derive(Clone, Copy)]
pub enum BuildingState {
    Solid,
    Burning,
    Destroyed,
}

// Potential state of each cell
#[derive(Clone, Copy)]
pub enum CellState {
    Empty,

    // Illager type and corresponding state
    IllagerClan(IllagerClan, IllagerState),

    // Villagers have no different state types
    VillagerClan(VillagerClan),

    // 0, 1
    // 2, 3
    House(BuildingState, u8),

    // 0, 1
    // 2, 3
    BigRock(u8),
    Rock,

    // 0
    // 1
    Lampost(u8),

    Bell,

    // 0, 1
    // 2, 3
    Tree(u8),

    // 0, 1
    // 2, 3
    Stand(u8),

    // todo: add other house (pen)

    // 0, 1
    // 2, 3
    // 4, 5
    Church(BuildingState, u8),

    // 0, 1
    Flowers(u8),
    
    // 0, 1
    Hay(u8),
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum Direction {
    N,  // Up
    E,  // Right
    S,  // Down
    W,  // Left
    NE, // Up-right
    SE, // Down-right
    NW, // Up-left
    SW, // Down-left
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
    tick: u8,
    cursors: [u8; 2],
    old_gamepad: [u8; 2],
    new_gamepad: [u8; 2],

    sheet: Sprite,

    //minions: VeryDumbSlotMap<MinionLink>,    

    current_selected_class: [u8; 2],
    grid: [CellState; 192],
}

impl Game {
    unsafe fn new() -> Self {
        // grey, beige, green, brown
        *PALETTE = [0xeacfb2, 0xc69478, 0x8a5543, 0x441d1f];

        // Read seed from disk and increment it, saving it again
        let mut seed = 0u64;
        diskr(
            (&mut seed as *mut u64).cast::<u8>(),
            std::mem::size_of::<u64>() as u32,
        );
        seed += 1;

        diskw(
            (&mut seed as *mut u64).cast::<u8>(),
            std::mem::size_of::<u64>() as u32,
        );
        fastrand::seed(seed);

        Self {
            villager: 9,
            tick: 0,
            illager: 9,
            cursors: [176, 191],
            //minions: VeryDumbSlotMap::default(),
            new_gamepad: [0; 2],
            old_gamepad: [*GAMEPAD1, *GAMEPAD2],
            current_selected_class: [0, 0],
            sheet: sprite!("../packed/sprite.pak"),
            grid: terrain::generate(),
        }
    }

    unsafe fn run(&mut self) {
        if self.tick == 0 {
            self.update();
        }

        self.fetch_input();
        self.draw_background();
        self.draw_sprites();
        self.draw_footer();
        self.draw_cursors();
        self.debug_palette();
        

        self.tick += 1;
        self.tick %= 60;
    }

    // Fetch gamepad input. Also works in multiplayer. Only supports 2 players
    // Also moves the appropriate selectors if necessary
    unsafe fn fetch_input(&mut self) {
        // Completely on the verge of breaking if bounds weren't hard coded lol
        fn move_cursor(dir: Direction, cursor: &mut u8) {
            let mut x = (*cursor % 16) as i32;
            let mut y = (*cursor / 16) as i32;

            match dir {
                Direction::N => y -= 1,
                Direction::S => y += 1,
                Direction::W => x -= 1,
                Direction::E => x += 1,

                _ => {}
            }

            x = x.rem_euclid(16);
            y = y.rem_euclid(12);

            *cursor = (y * 16 + x) as u8;
        }

        const GAMEPADS: [*const u8; 2] = [GAMEPAD1, GAMEPAD2];
        for index in 0..2 {
            let last = self.old_gamepad[index];
            let current = *GAMEPADS[index];
            let new = current & (last ^ current);
            self.old_gamepad[index] = current;
            self.new_gamepad[index] = new;

            // Move cursor on grid
            let x = self.new_gamepad[index];
            let grid_pos: &mut u8 = &mut self.cursors[index];
            if x & BUTTON_UP != 0 {
                move_cursor(Direction::N, grid_pos);
            } else if x & BUTTON_DOWN != 0 {
                move_cursor(Direction::S, grid_pos);
            } else if x & BUTTON_LEFT != 0 {
                move_cursor(Direction::W, grid_pos);
            } else if x & BUTTON_RIGHT != 0 {
                move_cursor(Direction::E, grid_pos);
            }

            let selected = &mut self.current_selected_class[index];

            // Cycle current selected class
            if new & BUTTON_2 != 0 {
                *selected += 1;
                *selected %= 3;
            }

            // Place currently selected class
            if new & BUTTON_1 != 0 {
                let points: &mut u8 = if index == 0 {
                    &mut self.villager
                } else {
                    &mut self.illager
                };

                // make sure the cell is empty so we can place our shit there
                if matches!(self.grid[*grid_pos as usize], CellState::Empty) {
                    // checked sub to make sure we don't cause a crash (also saves us from
                    // manually comparing to check if we have enough points to spend)
                    if let Some(new_points) =
                        points.checked_sub(PRICES[*selected as usize + 3 * index])
                    {
                        *points = new_points;

                        // logic that handles setting new classes
                        let cell = &mut self.grid[*grid_pos as usize];

                        // "index" is play index (where 0 is villager and 1 is illager)
                        // "selected" is the selected class index (0..3)
                        *cell = match (index, selected) {
                            // villager clan classes
                            (0, 0) => CellState::VillagerClan(VillagerClan::Villager),
                            (0, 1) => CellState::VillagerClan(VillagerClan::Farmer),
                            (0, 2) => {
                                CellState::VillagerClan(VillagerClan::Smith { golem_id: None })
                            }

                            // illager clan classes
                            (1, 0) => {
                                CellState::IllagerClan(IllagerClan::Vex { id: NonMaxU8::new(0).unwrap() }, IllagerState::Idle)
                            }
                            (1, 1) => {
                                CellState::IllagerClan(IllagerClan::Pillager, IllagerState::Idle)
                            }
                            (1, 2) => CellState::IllagerClan(
                                IllagerClan::Evoker {
                                    vex_ids: [None, None],
                                },
                                IllagerState::Idle,
                            ),

                            _ => unreachable!(),
                        };
                    }
                }
            }
        }
    }

    // Iterate on all pieces
    unsafe fn update(&mut self) {
        fn try_move(index: u8, dir: Direction, grid: &mut [CellState]) -> bool {
            let Some(index_2) = apply_direction(index, dir).map(|i| i as usize) else {
                //println!("FUCK!");
                return false;
            };
            if matches!(grid[index_2], CellState::Empty) {
                grid[index_2] = grid[index as usize];
                grid[index as usize] = CellState::Empty;
                return true;
            } else {
                return false;
            }
        }

        let mut suspicious_grid: [CellState; 192] = self.grid;
        let grid_ref: &mut [CellState; 192] = &mut suspicious_grid;

        for (_index, state) in self.grid.iter().enumerate() {
            match state {
                CellState::IllagerClan(id, _state) => match id {
                    IllagerClan::Vindicator => { try_move(_index as u8, random_direction(self.tick, 0..4), grid_ref); }
                    IllagerClan::Pillager => { try_move(_index as u8, random_direction(self.tick, 0..4), grid_ref); }
                    IllagerClan::Evoker { .. } => { try_move(_index as u8, random_direction(self.tick, 0..4), grid_ref); }
                    IllagerClan::Vex { .. } => { try_move(_index as u8, random_direction(self.tick, 0..8), grid_ref); }
                },

                CellState::VillagerClan(id) => match id {
                    VillagerClan::Villager => { try_move(_index as u8, random_direction(self.tick, 0..4), grid_ref); }
                    VillagerClan::Farmer => { try_move(_index as u8, random_direction(self.tick, 0..4), grid_ref); }
                    VillagerClan::Smith { .. } => { try_move(_index as u8, random_direction(self.tick, 0..4), grid_ref); }
                    VillagerClan::Golem { .. } => { try_move(_index as u8, random_direction(self.tick, 0..4), grid_ref); }
                },

                _ => continue,
            }
        }

        // Update the minion values for parent/minion position
        /*
        for (index, state) in self.grid.iter().enumerate() {
            match state {
                // Illager update link's minion pos
                CellState::IllagerClan(IllagerClan::Evoker { vex_ids }, _) => {
                    for id in vex_ids.iter().filter_map(|x| x.as_ref()) {
                        self.minions
                            .get_mut(id.get())
                            .unwrap()
                            .parent_position_index = index as u8;
                    }
                }

                // Vex update link's minion pos
                CellState::IllagerClan(IllagerClan::Vex { id }, _) => {
                    self.minions.get_mut(id.get()).unwrap().minion_position_index = index as u8;
                }

                // Smith update link's parent pos
                CellState::VillagerClan(VillagerClan::Smith { golem_id: Some(id) }) => {
                    self.minions
                        .get_mut(id.get())
                        .unwrap()
                        .parent_position_index = index as u8;
                }

                // Golem update link's minion pos
                CellState::VillagerClan(VillagerClan::Golem { id, .. }) => {
                    self.minions.get_mut(id.get()).unwrap().minion_position_index = index as u8;
                }

                _ => {}
            }
        }
        */

        self.grid = suspicious_grid;
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
        
    // Common functionality for rendering multi-sprite buildings (houses, church, bell, torch pole)
    // width and height are in sprite size (so for house this would be 2, 2)
    unsafe fn draw_multi_sprite(&self, index: u8, mega_width: u8, src_x: u32, src_y: u32, dst_x: i32, dst_y: i32) {
        let x_offset = (index % mega_width) as u32;
        let y_offset = (index / mega_width) as u32;

        let act_src_y = y_offset * 10 + src_y;
        let act_src_x = x_offset * 10 + src_x;

        blit_sub(
            &self.sheet.bytes,
            dst_x,
            dst_y,
            10,
            10,
            act_src_x,
            act_src_y,
            self.sheet.width,
            self.sheet.flags,
        )
    }

    // Draw a single sprite
    unsafe fn draw_sprite(&self, src_x: u32, src_y: u32, dst_x: i32, dst_y: i32) {
        blit_sub(
            &self.sheet.bytes,
            dst_x,
            dst_y,
            10,
            10,
            src_x,
            src_y,
            self.sheet.width,
            self.sheet.flags,
        )
    }


    // Draw the grid with the appropriate sprites
    unsafe fn draw_sprites(&self) {
        *DRAW_COLORS = 0b0100_0011_0010_0001;
        
        let sprite = self.sheet.bytes;
        let flags = self.sheet.flags;

        // Used for burning buildings
        let burning_x_offset = (self.tick < 30) as u32 * 20;

        for (index, state) in self.grid.iter().enumerate() {
            let (dst_grid_x, dst_grid_y) = vec_from_grid(index as u8);
            let (dst_grid_x, dst_grid_y) = (dst_grid_x as usize, dst_grid_y as usize);
            let dst_x = (dst_grid_x * 10) as i32;
            let dst_y = (dst_grid_y * 10) as i32;

            match state {
                CellState::IllagerClan(_type, state) => {
                    // src x pos inside the sprite sheet that we will blit from
                    let src_x = match _type {
                        IllagerClan::Vindicator => 0,
                        IllagerClan::Pillager => 10,
                        IllagerClan::Evoker { .. } => 20,
                        IllagerClan::Vex { .. } => 30,
                    };

                    // src y pos from the sprite sheet
                    let src_y = match state {
                        IllagerState::Idle => 0,
                        IllagerState::Action => 10,
                    };

                    blit_sub(
                        self.sheet.bytes,
                        dst_x,
                        dst_y,
                        10,
                        10,
                        src_x,
                        src_y,
                        self.sheet.width,
                        self.sheet.flags,
                    );
                }

                CellState::VillagerClan(_type) => {
                    // src x and src y positions inside the sprite sheet
                    let (src_x, src_y) = match _type {
                        VillagerClan::Villager => (40, 0),
                        VillagerClan::Farmer => (50, 0),
                        VillagerClan::Smith { .. } => (60, 0),
                        VillagerClan::Golem { state, .. } => match state {
                            GolemState::Attack => (60, 10),
                            GolemState::Broken => (70, 10),
                            GolemState::Idle => (70, 0),
                        },
                    };

                    blit_sub(
                        sprite,
                        dst_x,
                        dst_y,
                        10,
                        10,
                        src_x,
                        src_y,
                        self.sheet.width,
                        flags,
                    );
                }

                CellState::House(state, i) => {
                    let src_x = match state {
                        BuildingState::Solid => 0,
                        BuildingState::Burning => 20 + burning_x_offset,
                        BuildingState::Destroyed => 60,
                    };

                    self.draw_multi_sprite(*i, 2, src_x, 20, dst_x, dst_y);
                },

                CellState::BigRock(i) => self.draw_multi_sprite(*i, 2, 0, 40, dst_x, dst_y),
                CellState::Rock => self.draw_sprite(30, 40, dst_x, dst_y),
                CellState::Lampost(i) => self.draw_multi_sprite(*i, 1, 20, 40, dst_x, dst_y),
                CellState::Bell => self.draw_sprite(30, 50, dst_x, dst_y),
                CellState::Tree(i) => self.draw_multi_sprite(*i, 2, 40, 40, dst_x, dst_y),
                CellState::Stand(i) => self.draw_multi_sprite(*i, 2, 60, 40, dst_x, dst_y),
                CellState::Church(state, i) => {
                    let src_x = match state {
                        BuildingState::Solid => 0,
                        BuildingState::Burning => 20 + burning_x_offset,
                        BuildingState::Destroyed => 60,
                    };

                    self.draw_multi_sprite(*i, 2, src_x, 80, dst_x, dst_y);
                },
                CellState::Flowers(i) => self.draw_multi_sprite(*i, 2, 10, 110, dst_x, dst_y),
                CellState::Hay(i) => self.draw_multi_sprite(*i, 2, 30, 110, dst_x, dst_y),
                _ => continue,
            };
        }
    }

    // Draw the player cursors. Different colors assigned to each team
    unsafe fn draw_cursors(&self) {
        *DRAW_COLORS = 0b0100_0000_0000_0001;
        for (index, selector_position) in self.cursors.iter().enumerate() {
            let (posx, posy) = vec_from_grid(*selector_position);

            /*
            // TODO: Should we add the new cursors or keep it like the old ones (where you flip in code)
            // cursor is off center by 3 pixels to satisfy restriction that width must be divible by 8
            blit_sub(
                self.sheet.bytes,
                (posx * 10) as i32,
                (posy * 10) as i32,
                10,
                10,
                30,
                40 + index as u32 * 10,
                self.sheet.width,
                self.sheet.flags,
            );
            */

            const COLORS: [u8; 2] = [0b1000000, 0b0010000];
            //*DRAW_COLORS = COLORS[index] as u16;

            let flags = if index == 0 {
              BLIT_FLIP_X 
            } else {
                0
            } | self.sheet.flags;
            
            // cursor is off center by 3 pixels to satisfy restriction that width must be divible by 8
            blit_sub(&self.sheet.bytes, posx as i32 * 10, posy as i32 * 10, 10, 10, 70, 110, self.sheet.width, flags);
        
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

// Convert local coords to index
fn grid_from_vec(x: u8, y: u8) -> u8 {
    x + y * 16
}

// Convert index to local coords
fn vec_from_grid(index: u8) -> (u8, u8) {
    let x = index % 16;
    let y = index / 16;
    (x, y)
}

// Apply a direction in index based space
fn apply_direction(index: u8, dir: Direction) -> Option<u8> {
    let (x, y) = vec_from_grid(index);
    let (mut x, mut y) = (x as i8, y as i8);

    match dir {
        Direction::N => y -= 1,
        Direction::E => x += 1,
        Direction::S => y += 1,
        Direction::W => x -= 1,
        Direction::NE => {y -= 1; x += 1},
        Direction::SE => {y += 1; x += 1},
        Direction::NW => {y -= 1; x -= 1},
        Direction::SW => {y += 1; x -= 1},
    };

    ((0..12).contains(&y) && (0..16).contains(&x)).then(|| grid_from_vec(x as u8, y as u8))
}

unsafe fn random_direction(seed: u8, range: std::ops::Range<u8>) -> Direction {
    transmute::<u8, Direction>(fastrand::u8(range))
}

#[no_mangle]
unsafe fn start() {
    GAME = Some(Game::new());
}

#[no_mangle]
unsafe fn update() {
    GAME.as_mut().unwrap().run();
}
