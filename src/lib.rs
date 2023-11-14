mod alloc;
mod sprites;
mod terrain;
mod wasm4;
use std::mem::transmute;
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

// Main grid parameters
pub const GRID_SIZE_X: u8 = 30;
pub const GRID_SIZE_Y: u8 = 30;
pub const AREA: usize = GRID_SIZE_X as usize * GRID_SIZE_Y as usize;
pub const GRID_LOCAL_SIZE_X: u8 = 16;
pub const GRID_LOCAL_SIZE_Y: u8 = 12;

// Village stuff goes first since P1 is controlling the villagers
const PRICES: [u8; 6] = [VILLAGER, FARMER, SMITH, VINDICATOR, PILLAGER, EVOKER];

// Entities associated with illagers (vex included)
#[derive(Clone, Copy)]
pub enum IllagerClan {
    Vindicator,
    Pillager,
    Evoker(u8),
    Vex(u8),
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

// Entities associated with villagers (golems included)
#[derive(Clone, Copy)]
pub enum VillagerClan {
    Villager,
    Farmer,
    Smith(u8),
    Golem(u8, GolemState),
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
    House2(BuildingState, u8),

    // 0, 1
    // 2, 3
    BigRock(u8),
    Rock,

    // 0
    // 1
    Lamppost(u8),

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
    Farm(u8),
    
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
    seed: u64,
    villager: u8,
    illager: u8,
    tick: u8,
    cursors: [u16; 2],
    old_gamepad: [u8; 2],
    new_gamepad: [u8; 2],
    cursor_timer: [u8; 2],
    view_local_cameras: [(u8, u8); 2],
    current_player: u8,
    
    sheet: Sprite,
    current_selected_class: [u8; 2],
    grid: Box<[CellState; AREA as usize]>,
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
        let grid = terrain::generate();
        
        Self {
            seed,
            villager: 9,
            tick: 0,
            illager: 9,
            cursors: [0, 0],
            current_player: 0, 
            new_gamepad: [0; 2],
            old_gamepad: [*GAMEPAD1, *GAMEPAD2],
            cursor_timer: [0, 0],
            current_selected_class: [0, 0],
            sheet: sprite!("../packed/sprite.pak"),
            grid,
            view_local_cameras: [(0, 0), (0, 0)],
        }
    }

    unsafe fn run(&mut self) {
        if (*NETPLAY >> 2) == 0 {
            text("Mohsin cannot ", 0, 0);
            text("play alone....", 0, 10);
        } else {
            self.current_player = *NETPLAY & 0b11;

            if self.tick == 0 {
                self.update();
            }
    
            self.fetch_input();
            self.draw_background();
            self.draw_sprites();
            self.draw_footer();
            self.draw_cursors();
            //self.debug_palette();

            self.tick += 1;
            self.tick %= 60;
        }
    }

    // Fetch gamepad input. Also works in multiplayer. Only supports 2 players
    // Also moves the appropriate selectors (and current player view if needed)
    unsafe fn fetch_input(&mut self) {
        // Moves the cursor, also moving the view local camera when it goes out of bounds
        fn move_cursor(dir: Direction, cursor: &mut u16, camera: &mut (u8, u8)) {
            let mut x = (*cursor % GRID_SIZE_X as u16) as i8;
            let mut y = (*cursor / GRID_SIZE_X as u16) as i8;

            match dir {
                Direction::N => y -= 1,
                Direction::S => y += 1,
                Direction::W => x -= 1,
                Direction::E => x += 1,
                _ => {}
            }

            let x = x.clamp(0, GRID_SIZE_X as i8-1) as u8;
            let y = y.clamp(0, GRID_SIZE_Y as i8-1) as u8;

            match x.checked_sub(camera.0) {
                Some(x) if x >= GRID_LOCAL_SIZE_X => camera.0 += 1,
                None => camera.0 -= 1,
                _ => {}
            }

            match y.checked_sub(camera.1) {
                Some(y) if y >= GRID_LOCAL_SIZE_Y => camera.1 += 1,
                None => camera.1 -= 1,
                _ => {}
            }

            *cursor = y as u16 * GRID_SIZE_X as u16 + x as u16;
        }

        const GAMEPADS: [*const u8; 2] = [GAMEPAD1, GAMEPAD2];
        for index in 0..2 {
            let last = self.old_gamepad[index];
            let current = *GAMEPADS[index];
            let new = current & (last ^ current);
            self.old_gamepad[index] = current;
            self.new_gamepad[index] = new;

            // Move cursor on grid
            let grid_pos: &mut u16 = &mut self.cursors[index];
            let camera = &mut self.view_local_cameras[index];
            let tick: &mut u8 = &mut self.cursor_timer[index];
            let tick_check: bool = *tick % 5 == 0;
            *tick = tick.wrapping_add(1);
            if current & BUTTON_UP != 0 {
                if tick_check { move_cursor(Direction::N, grid_pos, camera)};
            } else if current & BUTTON_DOWN != 0 {
                if tick_check { move_cursor(Direction::S, grid_pos, camera)};
            } else if current & BUTTON_LEFT != 0 {
                if tick_check { move_cursor(Direction::W, grid_pos, camera)};
            } else if current & BUTTON_RIGHT != 0 {
                if tick_check { move_cursor(Direction::E, grid_pos, camera)};
            } else {
                *tick = 0;
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
                                CellState::VillagerClan(VillagerClan::Smith(0))
                            }

                            // illager clan classes
                            (1, 0) => {
                                CellState::IllagerClan(IllagerClan::Vindicator, IllagerState::Idle)
                            }
                            (1, 1) => {
                                CellState::IllagerClan(IllagerClan::Pillager, IllagerState::Idle)
                            }
                            (1, 2) => CellState::IllagerClan(
                                IllagerClan::Evoker(0),
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
    fn update(&mut self) {
        fn try_move(index: u16, dir: Direction, grid: &mut [CellState]) -> bool {
            let Some(index_2) = apply_direction(index, dir).map(|i| i as usize) else {
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

        let mut suspicious_grid: Box<[CellState; AREA]> = self.grid.clone();
        let grid_ref: &mut [CellState; AREA] = &mut suspicious_grid;

        for (_index, state) in self.grid.iter().enumerate() {
            match state {
                CellState::IllagerClan(id, _state) => match id {
                    IllagerClan::Vindicator => { try_move(_index as u16, random_direction(), grid_ref); }
                    IllagerClan::Pillager => { try_move(_index as u16, random_direction(), grid_ref); }
                    IllagerClan::Evoker { .. } => { try_move(_index as u16, random_direction(), grid_ref); }
                    IllagerClan::Vex { .. } => { try_move(_index as u16, random_direction_with_diagonal(), grid_ref); }
                },

                CellState::VillagerClan(id) => match id {
                    VillagerClan::Villager => { try_move(_index as u16, random_direction(), grid_ref); }
                    VillagerClan::Farmer => { try_move(_index as u16, random_direction(), grid_ref); }
                    VillagerClan::Smith { .. } => { try_move(_index as u16, random_direction(), grid_ref); }
                    VillagerClan::Golem { .. } => { try_move(_index as u16, random_direction(), grid_ref); }
                },

                _ => continue,
            }
        }

        self.grid = suspicious_grid;
    }

    // Draw a footer containing points, classes to summon, and current selected cell
    unsafe fn draw_footer(&mut self) {

        *DRAW_COLORS = 0b0100000;
        rect(0, 120, 160, 35);

        *DRAW_COLORS = 0b1000100;
        rect(0, 155, 160, 5);

        *DRAW_COLORS = 0b0100_0000_0000_0100;
        let mut buffer = itoa::Buffer::new();
        text(buffer.format(self.villager), 71, 135);

        *DRAW_COLORS = 0b0100_0011_0010_0001;

        // Draw class portraits - width 17, height 27
        for x in 0..3 {
            self.draw_sprite(
                4 + 19 * x,
                124,
                17,
                27,
                0 + 17 * x as u32,
                120 + self.current_player as u32 * 27
            )
        }

        // Draw action buttons - width 9, height 9
        for x in 0..3 {
            self.draw_sprite(61 + 11 * x, 124, 9, 9, 51, 147 + 9 * x as u32)
        }

        // Draw villager and emerald symbols (text above)
        self.draw_sprite(62, 135, 6, 7, 51, 123);
        self.draw_sprite(62, 144, 6, 7, 51, 130);


        // Draw selection cursor todo

        // Draw log? todo
    }

    fn draw_sprite(&self, x: i32, y: i32, width: u32, height: u32, src_x: u32, src_y: u32) {
        blit_sub(
            &self.sheet.bytes,
            x,
            y,
            width,
            height,
            src_x,
            src_y,
            self.sheet.width,
            self.sheet.flags,
        );
    }

    // Draw the background color
    fn draw_background(&self) {
        unsafe { *DRAW_COLORS = 0b0011_0011_0011_0011; }
        rect(0, 0, 160, 120);
    }
        
    // Common functionality for rendering multi-sprite buildings (houses, church, bell, torch pole)
    // width and height are in sprite size (so for house this would be 2, 2)
    fn draw_multi_grid_sprite(&self, index: u8, mega_width: u8, src_x: u32, src_y: u32, dst_x: i32, dst_y: i32) {
        let x_offset = (index % mega_width) as u32;
        let y_offset = (index / mega_width) as u32;

        let act_src_y = y_offset * 10 + src_y;
        let act_src_x = x_offset * 10 + src_x;

        self.draw_grid_sprite(act_src_x, act_src_y, dst_x, dst_y)
    }

    // Util function for grid sprites only
    fn draw_grid_sprite(&self, src_x: u32, src_y: u32, dst_x: i32, dst_y: i32) {
        self.draw_sprite(dst_x, dst_y, 10, 10, src_x, src_y)
    }

    // Draw a background grass sprite before rendering other sprites
    fn draw_background_grass(&self, base: (u8, u8), offset: (u8, u8), dst: (i32, i32)) {
        let (x, flip, variant) = {
            let a = ((base.0 + offset.0) as u64 + (self.seed.wrapping_mul(0x9E3779B97F4A7C15) % 1684)).wrapping_mul(0x4a9b41c68d);
            let b = ((base.1 + offset.1) as u64 + (self.seed.wrapping_mul(0x6c7967656e657261) % 6475)).wrapping_mul(0x94ba7c6d9b);
            let t = 0xffffffffu32 as f32;
            let hash = ((((a ^ b) as f32) / t) * 10.0) as u32;
            (hash % 4, ((hash % 16 > 8) as u32) << 1, ((hash % 10 < 3) as u32))
        };

        if x == 0 {
            blit_sub(
                &self.sheet.bytes,
                dst.0,
                dst.1,
                10,
                10,
                50 + variant * 10,
                110,
                self.sheet.width,
                self.sheet.flags | flip,
            );
        }
    }


    // Draw the grid with the appropriate sprites
    unsafe fn draw_sprites(&mut self) {

        *DRAW_COLORS = 0b0100_0011_0010_0001;

        // Used for burning buildings
        let burning_x_offset = (self.tick < 30) as u32 * 20;

        for base_x in 0..GRID_LOCAL_SIZE_X {
            for base_y in 0..GRID_LOCAL_SIZE_Y {
                let (offset_x, offset_y) = self.view_local_cameras[self.current_player as usize];
                let state = &self.grid[grid_from_vec(base_x + offset_x, base_y + offset_y) as usize];
                let dst_x = (base_x * 10) as i32;
                let dst_y = (base_y * 10) as i32;

                self.draw_background_grass((base_x, base_y), (offset_x, offset_y), (dst_x, dst_y));

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
                    
                        self.draw_grid_sprite(src_x, src_y, dst_x, dst_y)
                    }

                    CellState::VillagerClan(_type) => {
                        // src x and src y positions inside the sprite sheet
                        let (src_x, src_y) = match _type {
                            VillagerClan::Villager => (40, 0),
                            VillagerClan::Farmer => (50, 0),
                            VillagerClan::Smith { .. } => (60, 0),
                            VillagerClan::Golem(_, state) => match state {
                                GolemState::Attack => (60, 10),
                                GolemState::Broken => (70, 10),
                                GolemState::Idle => (70, 0),
                            },
                        };

                        self.draw_grid_sprite(src_x, src_y, dst_x, dst_y);
                    }

                    CellState::House(state, i) => {
                        let src_x = match state {
                            BuildingState::Solid => 0,
                            BuildingState::Burning => 20 + burning_x_offset,
                            BuildingState::Destroyed => 60,
                        };

                        self.draw_multi_grid_sprite(*i, 2, src_x, 20, dst_x, dst_y);
                    },

                    CellState::House2(state, i) => {
                        let src_x = match state {
                            BuildingState::Solid => 0,
                            BuildingState::Burning => 20 + burning_x_offset,
                            BuildingState::Destroyed => 60,
                        };

                        self.draw_multi_grid_sprite(*i, 2, src_x, 60, dst_x, dst_y);
                    },

                    CellState::BigRock(i) => self.draw_multi_grid_sprite(*i, 2, 0, 40, dst_x, dst_y),
                    CellState::Rock => self.draw_grid_sprite(30, 40, dst_x, dst_y),
                    CellState::Lamppost(i) => self.draw_multi_grid_sprite(*i, 1, 20, 40, dst_x, dst_y),
                    CellState::Bell => self.draw_grid_sprite(30, 50, dst_x, dst_y),
                    CellState::Tree(i) => self.draw_multi_grid_sprite(*i, 2, 40, 40, dst_x, dst_y),
                    CellState::Stand(i) => self.draw_multi_grid_sprite(*i, 2, 60, 40, dst_x, dst_y),
                    CellState::Church(state, i) => {
                        let src_x = match state {
                            BuildingState::Solid => 0,
                            BuildingState::Burning => 20 + burning_x_offset,
                            BuildingState::Destroyed => 60,
                        };
                        self.draw_multi_grid_sprite(*i, 2, src_x, 80, dst_x, dst_y);
                    },
                    CellState::Farm(i) => self.draw_multi_grid_sprite(*i, 2, 10, 110, dst_x, dst_y),
                    CellState::Hay(i) => self.draw_multi_grid_sprite(*i, 2, 30, 110, dst_x, dst_y),
                    _ => continue,
                }
            }
        }
    }

    // Draw the player cursors. Different colors assigned to each team
    unsafe fn draw_cursors(&self) {
        *DRAW_COLORS = 0b0100_0000_0000_0001;
        let index = self.current_player as usize;
        let (mut posx, mut posy) = vec_from_grid(self.cursors[index]);
        posx -= self.view_local_cameras[index].0;
        posy -= self.view_local_cameras[index].1;

        const COLORS: [u8; 2] = [0b1000000, 0b0010000];
        *DRAW_COLORS = COLORS[index] as u16;

        let flags = if index == 0 {
            BLIT_FLIP_X 
        } else {
            0
        } | self.sheet.flags;
        
        // cursor is off center by 3 pixels to satisfy restriction that width must be divible by 8
        blit_sub(&self.sheet.bytes, posx as i32 * 10, posy as i32 * 10, 10, 10, 0, 0, self.sheet.width, flags);
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
fn grid_from_vec(x: u8, y: u8) -> u16 {
    x as u16 + y as u16 * (GRID_SIZE_X as u16)
}

// Convert index to local coords
fn vec_from_grid(index: u16) -> (u8, u8) {
    let x = index % (GRID_SIZE_X as u16);
    let y = index / (GRID_SIZE_X as u16);
    (x as u8, y as u8)
}

// Apply a direction in index based space
fn apply_direction(index: u16, dir: Direction) -> Option<u16> {
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

    let (x, y) = (x as u8, y as u8);
    ((0..GRID_SIZE_Y).contains(&y) && (0..GRID_SIZE_X).contains(&x)).then(|| grid_from_vec(x as u8, y as u8))
}

fn random_direction() -> Direction {
    unsafe { transmute::<u8, Direction>(fastrand::u8(0..4)) }
}

fn random_direction_with_diagonal() -> Direction {
    unsafe { transmute::<u8, Direction>(fastrand::u8(0..8)) }
}

#[no_mangle]
unsafe fn start() {
    GAME = Some(Game::new());
}

#[no_mangle]
unsafe fn update() {
    GAME.as_mut().unwrap().run();
}
