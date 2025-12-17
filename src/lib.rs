mod alloc;
mod sprites;
mod terrain;
mod wasm4;
pub use sprites::*;
use std::{mem::transmute, cell::Cell, ops::Range};
use wasm4::*;
mod sound;
pub use sound::*;

static mut GAME: Option<Game> = None;

// Debug constants
const DEBUG_PALETTE: bool = false;
const MULTIPLAYER: bool = false;

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
type Board = Box<[CellState; AREA]>;
pub const GRID_LOCAL_SIZE_X: u8 = 16;
pub const GRID_LOCAL_SIZE_Y: u8 = 12;

// Gameplay constant
pub const CURSOR_MOVEMENT_SPEED_INV: u8 = 7;

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

    // 0, 1
    // 2, 3
    // 4, 5
    Church(BuildingState, u8),

    // 0, 1
    // todo: add timer for being farmed
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
    emeralds: [u8; 2],
    tick: u8,
    cursors: [u16; 2],
    old_gamepad: [u8; 2],
    new_gamepad: [u8; 2],
    cursor_timer: [u8; 2],
    view_local_cameras: [(u8, u8); 2],
    current_player: u8,
    button_held: [bool; 2],
    action_possible: [bool; 2],

    sheet: Sprite,
    current_selected_class: [u8; 2],
    goals: [u8; 4],
    grid: Board,
}

#[derive(Clone, Copy)]
enum Color {
    Lightest,
    Lighter,
    Darker,
    Darkest,
    Transparent
}

impl Color {
    fn to_raw(self) -> u8 {
        match self {
            Color::Lightest => 1,
            Color::Lighter => 2,
            Color::Darker => 3,
            Color::Darkest => 4,
            Color::Transparent => 0,
        }
    }
}

impl Game {
    unsafe fn new() -> Self {
        // lightest, lighter, darker, darkest
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

        // the villager camera always starts at the middle of the map (village)
        let (mid_x, mid_y) = (GRID_SIZE_X / 2, GRID_SIZE_Y / 2);
        let village_cursor = grid_from_vec(mid_x, mid_y);


        Self {
            seed,
            emeralds: [200, 100],
            tick: 0,
            cursors: [village_cursor, 0],
            current_player: 0,
            button_held: [false, false],
            action_possible: [false, false],
            new_gamepad: [0; 2],
            old_gamepad: [*GAMEPAD1, *GAMEPAD2],
            cursor_timer: [0, 0],
            current_selected_class: [0, 0],
            goals: [u8::MAX; 4],
            sheet: sprite!("../packed/sprite.pak"),
            grid,
            view_local_cameras: [(mid_x - GRID_LOCAL_SIZE_X / 2, mid_y - GRID_LOCAL_SIZE_Y / 2), (0, 0)],
        }
    }

    unsafe fn set_text_colors(text_color: Color, background_color: Color) {
        *DRAW_COLORS = ((text_color.to_raw()) | (background_color.to_raw() << 4)) as u16;
    }

    unsafe fn set_rect_colors(infill_color: Color, outline_color: Color) {
        *DRAW_COLORS = ((infill_color.to_raw()) | (outline_color.to_raw() << 4)) as u16;
    }

    unsafe fn run(&mut self) {
        if (*NETPLAY >> 2) == 0 && MULTIPLAYER {
            *DRAW_COLORS = 0b0100_0000_0000_0100;
            text("Waiting for", 36, 20);
            text("player 2", 46, 30);
            text("and mohsin", 38, 40);
            *DRAW_COLORS = 0b0100_0011_0010_0001;
            self.draw_sprite(40, 8, 80, 10, 0, 180)
        } else {
            if MULTIPLAYER {
                self.current_player = *NETPLAY & 0b11;
            } else {
                self.current_player = 0;
            }

            if self.tick == 0 {
                self.update();
            }

            self.action_possible();
            self.fetch_input();
            self.draw_background();
            self.draw_sprites();
            self.draw_footer();
            self.draw_cursors();
            self.debug_minimap();

            if (DEBUG_PALETTE) {
                self.debug_palette();
            }

            self.tick += 1;
            self.tick %= 60;
        }
    }

    //
    //
    // "SUMMON" TYPE ACTIONS: 0..=2
    // they summon:
    //    index       |     0       |    1     |    2
    // villager clan  |  villager   | farmer   | smith      
    // illager clan   |  vindicator | pillager | evoker     
    //
    //
    fn action_possible(&mut self) {
        // === Villagers ===
        // 1, 2, 3 & on house? true else false
        // 4 on anything? true else false
        // 5 on empty? true else false
        // 6 ? true
        // === Illagers ===
        // 1, 2, 3 & on empty border ? true else false
        // 4 on anything? true else false
        // 5 on empty? true else false
        // 6 ? true

        fn at_border(index: u16) -> bool {
            let (x, y) = vec_from_grid(index);
            if x == 0 || x == GRID_SIZE_X - 1 || y == 0 || y == GRID_SIZE_Y - 1 {
                true
            } else {
                false
            }
        }

        // If we are controlling Villagers, we want to be able to summon villagers when doing the "Action" when we have a building selected
        fn can_we_spawn_villagers(cell: &CellState, cursor: u16) -> bool {
            matches!(*cell, CellState::House(_, _) | CellState::Church(_, _) | CellState::House2(_, _))
        }

        // If we are controlling Illagers, we want to be able to summon them at the border of the map, and on empty cells as well
        fn can_we_spawn_illagers(cell: &CellState, cursor: u16) -> bool {
            matches!(*cell, CellState::Empty) && at_border(cursor)
        }
        
        for player_index in 0..2 {
            // calculate the cursor position and cell of the current player
            let cursor = self.cursors[player_index] as u16;
            let cell = &self.grid[cursor as usize];

            self.action_possible[player_index] = match self.current_selected_class[player_index] {
                // actions 0..=2 are the "summon type actions"
                // they summon:
                //    index       |     0       |    1     |    2
                // villager clan  |  villager   | farmer   | smith      
                // illager clan   |  vindicator | pillager | evoker     
                0..=2 => {
                    // check if player has enough currency (them emmies)
                    let player_has_enough_currency = self.emeralds[player_index] >= self.current_selected_class[player_index];

                    // check if the player can do the specified action
                    let player_can_do_thing = if player_index == 0 {
                        can_we_spawn_villagers(cell, cursor)
                    } else  {
                        can_we_spawn_illagers(cell, cursor)
                    };

                    // check if we can do both...
                    player_has_enough_currency && player_can_do_thing
                }

                3 | 5 => {
                    true
                }

                4 => {
                    if matches!(*cell, CellState::Empty) {
                        true
                    } else {
                        false
                    }
                }

                _ => unreachable!()
            };
        }
    }

    // Fetch gamepad input. Also works in multiplayer. Only supports 2 players
    // Also moves the appropriate selectors (and current player view if needed)
    unsafe fn fetch_input(&mut self) {
        // Moves the cursor, also moving the view local camera when it goes out of bounds
        fn move_cursor(step_x: i8, step_y: i8, cursor: &mut u16, camera: &mut (i8, i8)) {
            let mut x = (*cursor % GRID_SIZE_X as u16) as i8;
            let mut y = (*cursor / GRID_SIZE_X as u16) as i8;

            // Works ig
            x += (x + step_x).clamp(0, GRID_SIZE_X as i8 - 1) - x;
            y += (y + step_y).clamp(0, GRID_SIZE_Y as i8 - 1) - y;
            let x = x as u8;
            let y = y as u8;


            fn uncontained_clamp(coord: &mut i8, val: i8, max: u8) {                
                // local view min and max
                let min = *coord;
                let max = *coord + (max - 1) as i8;
                
                // max if x > max
                // min if x < min
                // val otherwise (to make subtraction 0)
                let diff = if val < min {
                    min
                } else if val > max {
                    max
                } else {
                    val
                };

                // move camera coordinate by difference
                *coord += val - diff;
            }

            // Completely refactored camera movement code that will simply try to accodomate keeping the cursor in the 16x12 grid
            // by taking cur diff to local edges
            uncontained_clamp(&mut camera.0, x as i8, GRID_LOCAL_SIZE_X);
            uncontained_clamp(&mut camera.1, y as i8, GRID_LOCAL_SIZE_Y);

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
            let cursor_tick_check: bool = *tick % CURSOR_MOVEMENT_SPEED_INV == 0;
            *tick = tick.wrapping_add(1);

            let mut step_left: i8 = -1;
            let mut step_right: i8 = 1;
            let mut step_down: i8 = 1;
            let mut step_up: i8 = -1;

            match &self.grid[*grid_pos as usize] {
                CellState::House(_x, y) | CellState::House2(_x, y) => {
                    if y % 2 == 0 {
                        step_right *= 2
                    } else {
                        step_left *= 2
                    };
                    if y > &1 {
                        step_up *= 2
                    } else {
                        step_down *= 2
                    };
                }

                CellState::Church(_x, y) => {
                    if y % 2 == 0 {
                        step_right *= 2
                    } else {
                        step_left *= 2
                    };
                    if y > &3 {
                        step_up *= 3
                    } else if y > &1 {
                        step_up *= 2;
                        step_down *= 2
                    } else {
                        step_down *= 3
                    };
                }

                CellState::BigRock(y) | CellState::Tree(y) | CellState::Stand(y) => {
                    if y % 2 == 0 {
                        step_right *= 2
                    } else {
                        step_left *= 2
                    };
                    if y > &1 {
                        step_up *= 2
                    } else {
                        step_down *= 2
                    };
                }

                CellState::Lamppost(y) => {
                    if y == &1 {
                        step_up *= 2
                    } else {
                        step_down *= 2
                    };
                }

                CellState::Farm(y) | CellState::Hay(y) => {
                    if y % 2 == 0 {
                        step_right *= 2
                    } else {
                        step_left *= 2
                    };
                }

                _ => { }
            }

            if cursor_tick_check && current != 0 {
                let (x, y) = if current & BUTTON_UP != 0 {
                    (0, step_up)
                } else if current & BUTTON_DOWN != 0 {
                    (0, step_down)
                } else if current & BUTTON_LEFT != 0 {
                    (step_left, 0)
                } else if current & BUTTON_RIGHT != 0 {
                    (step_right, 0)
                } else {
                    (0, 0)
                };

                let mut camera_temp = (camera.0 as i8, camera.1 as i8);
                move_cursor(x, y, grid_pos, &mut camera_temp);
                camera.0 = camera_temp.0 as u8;
                camera.1 = camera_temp.1 as u8;
            }
            
            if current == 0 {
                *tick = 0;
            }

            let selected = &mut self.current_selected_class[index];

            // Cycle current selected class
            if new & BUTTON_2 != 0 {
                *selected += 1;
                *selected %= 6;
                play_me_some_tones______boy(Noise::TungTungTungSahour);
            }

            // Place currently selected class
            // Action possible permits this to happen,
            // This is determined somewhere else (TBD)
            if new & BUTTON_1 != 0 && self.action_possible[index] {
                trace("player is doing action!");
                let points: &mut u8 = &mut self.emeralds[index];

                // make sure the cell is empty so we can place our shit there
                // this needs to be redone as villagers have to be placed by selecting a house and will come out of the bottom
                if *selected < 3 {
                    // checked sub to make sure we don't cause a crash (also saves us from
                    // manually comparing to check if we have enough points to spend)
                    if let Some(new_points) =
                        points.checked_sub(PRICES[*selected as usize + 3 * index])
                    {
                        // logic that handles setting new classes
                        // this makes things so much easier lol nice
                        let cell: Option<&mut CellState> = if index == 1 && matches!(self.grid[*grid_pos as usize], CellState::Empty) { 
                            Some(&mut self.grid[*grid_pos as usize])
                        } else {
                            // pick a plausible spawning position on the outline of the building
                            trace("pick spawn location...");
                            let random_position_building_outline: Option<u16> = match self.grid[*grid_pos as usize] {
                                CellState::Church(BuildingState::Solid, j) => pick_random_location_building_outline(
                                    &self.grid,
                                    2,
                                    3,
                                    j, 
                                    *grid_pos
                                ),

                                CellState::House(BuildingState::Solid, j) | CellState::House2(BuildingState::Solid, j) => pick_random_location_building_outline(
                                    &self.grid,
                                    2,
                                    2,
                                    j, 
                                    *grid_pos
                                ),

                                _ => None
                            };

                            random_position_building_outline.map(|position| &mut self.grid[position as usize])
                        };

                        // "index" is play index (where 0 is villager and 1 is illager)
                        // "selected" is the selected class index (0..3)
                        if let Some(cell_2) = cell { 
                            *points = new_points;
                            *cell_2 = match (index, selected) {
                                // villager clan classes
                                (0, 0) => CellState::VillagerClan(VillagerClan::Villager),
                                (0, 1) => CellState::VillagerClan(VillagerClan::Farmer),
                                (0, 2) => CellState::VillagerClan(VillagerClan::Smith(0)),

                                // illager clan classes
                                (1, 0) => CellState::IllagerClan(IllagerClan::Vindicator, IllagerState::Idle),
                                (1, 1) => CellState::IllagerClan(IllagerClan::Pillager, IllagerState::Idle),
                                (1, 2) => CellState::IllagerClan(IllagerClan::Evoker(0), IllagerState::Idle),                                

                                _ => CellState::Empty,
                            };

                            play_me_some_tones______boy(Noise::Ting);
                        } else {
                            play_me_some_tones______boy(Noise::SixSeven);
                        }
                    }
                } else {

                }
            }

            self.button_held[index] = current & BUTTON_1 != 0;
        }
    }

    // Iterate on all pieces
    fn update(&mut self) {
        fn swap_cells(src: u16, dst: u16, grid: &mut [CellState]) {
            let tmp = grid[src as usize];
            grid[src as usize] = grid[dst as usize];
            grid[dst as usize] = tmp;
        }

        fn try_move(index: u16, index_objective: u16, grid: &mut [CellState]) -> bool {
            let dir = random_direction_with_diagonal();
            if let Some(new_pos) = apply_direction(index, dir) {
                if matches!(grid[new_pos as usize], CellState::Empty) {
                    swap_cells(index, new_pos, grid);
                }
            }

            false
            /*
            if matches!(grid[index_objective as usize], CellState::Empty) {
                grid[index_objective as usize] = grid[index as usize];
                grid[index as usize] = CellState::Empty;
                true
            } else {
                false
            }
            */

            //false
        }

        fn pathfind(index: u16, index_objective: u16, grid: &mut [CellState]) -> u16 {
            let mut queue_list: Vec<u16> = Vec::new();
            let mut current_score: u16 = 0;
            let mut current = index;
            
            while current != index_objective {
                current = match queue_list.pop() {
                    None => break,
                    Some(i) => i,
                };
                
                for neighbour in get_neighbours(current).iter() {
                    
                }
    
                current_score += 1;
    
                break;
            }
            
            return 0;
        }

        let mut suspicious_grid: Box<[CellState; AREA]> = self.grid.clone();
        let grid_ref: &mut [CellState; AREA] = &mut suspicious_grid;

        for (_index, state) in self.grid.iter().enumerate() {
            match state {
                CellState::IllagerClan(id, _state) => match id {
                    IllagerClan::Vindicator => {
                        // check first for primary objectives (killing)
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                    IllagerClan::Pillager => {
                        // check first for primary objectives (shooting or pillaging)
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                    IllagerClan::Evoker { .. } => {
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                    IllagerClan::Vex { .. } => {
                        // just wander around or kill lol
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                },

                CellState::VillagerClan(id) => match id {
                    VillagerClan::Villager => {
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                    VillagerClan::Farmer => {
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                    VillagerClan::Smith { .. } => {
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                    VillagerClan::Golem { .. } => {
                        try_move(_index as u16, pathfind(_index as u16, 0, grid_ref), grid_ref);
                    }
                },

                _ => continue,
            }
        }

        self.grid = suspicious_grid;
    }

    // Draw a footer containing points, classes to summon, and current selected cell
    unsafe fn draw_footer(&mut self) {
        let class = self.current_selected_class[self.current_player as usize];
        let mut clan_entity_counts: [u16; 2] = [0; 2];

        for cell in self.grid.iter() {
            match cell {
                CellState::IllagerClan(_, _) => { clan_entity_counts[1] += 1 },
                CellState::VillagerClan(_) => { clan_entity_counts[0] += 1 },
                _ => {},
            }
        }
        
        Self::set_rect_colors(Color::Lightest, Color::Darkest);
        rect(0, 120, 160, 35);

        Self::set_rect_colors(Color::Darkest, Color::Darkest);
        rect(0, 155, 160, 5);

        Self::set_text_colors(Color::Darkest, Color::Lightest);
        let mut buffer = itoa::Buffer::new();
        text(
            buffer.format(self.emeralds[self.current_player as usize]),
            71,
            135,
        );

        // DEBUG!!!!!!!!!!!! This shows currently selected class lol (not meant to)
        text(buffer.format(clan_entity_counts[self.current_player as usize]), 71, 144);

        *DRAW_COLORS = 0b0100_0011_0010_0001;
        let button = self.button_held[self.current_player as usize];

        // Draw class portraits - width 17, height 27
        for x in 0..3 {
            let offset: u8 = if class == x as u8 && !button { 0 } else { 1 };
            self.draw_sprite(
                4 + 19 * x + offset as i32,
                124 + offset as i32,
                17 - offset as u32,
                27 - offset as u32,
                17 * x as u32,
                120 + self.current_player as u32 * 27,
            )
        }

        // Draw action buttons - width 9, height 9
        for x in 0..3 {
            let offset: i32 = if class == x as u8 + 3 && !button {
                0
            } else {
                1
            };
            self.draw_sprite(
                61 + 11 * x + offset,
                124 + offset,
                9 - offset as u32,
                9 - offset as u32,
                51,
                147 + 9 * x as u32,
            )
        }

        // Draw villager and emerald symbols (text above)
        self.draw_sprite(62, 135, 6, 7, 51, 123);
        self.draw_sprite(62, 144, 6, 7, 51, 130);

        // Draw log? todo
    }

    // Draw a general sprite from the main sprite sheet
    fn draw_sprite(&self, x: i32, y: i32, width: u32, height: u32, src_x: u32, src_y: u32) {
        blit_sub(
            self.sheet.bytes,
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
        unsafe {
            Self::set_rect_colors(Color::Darker, Color::Darker);
            //*DRAW_COLORS = 0b0011_0011_0011_0011;
        }
        rect(0, 0, 160, 120);
    }

    // Common functionality for rendering multi-sprite buildings (houses, church, bell, torch pole)
    // mega_width are in bigsprite size (so for house this would be 2)
    fn draw_multi_grid_sprite(
        &self,
        index: u8,
        mega_width: u8,
        src_x: u32,
        src_y: u32,
        dst_x: i32,
        dst_y: i32,
    ) {
        let x_offset = (index % mega_width) as u32;
        let y_offset = (index / mega_width) as u32;
        self.draw_grid_sprite(x_offset * 10 + src_x, y_offset * 10 + src_y, dst_x, dst_y)
    }

    // Util function for grid sprites only
    fn draw_grid_sprite(&self, src_x: u32, src_y: u32, dst_x: i32, dst_y: i32) {
        self.draw_sprite(dst_x, dst_y, 10, 10, src_x, src_y)
    }

    // Draw a background grass sprite before rendering other sprites
    fn draw_background_grass(&self, base: (u8, u8), offset: (u8, u8), dst: (i32, i32)) {
        let (x, flip, variant) = {
            let a = ((base.0 + offset.0) as u64
                + (self.seed.wrapping_mul(0x9E3779B97F4A7C15) % 1684))
                .wrapping_mul(0x4a9b41c68d);
            let b = ((base.1 + offset.1) as u64
                + (self.seed.wrapping_mul(0x6c7967656e657261) % 6475))
                .wrapping_mul(0x94ba7c6d9b);
            let t = 0xffffffffu32 as f32;
            let hash = ((((a ^ b) as f32) / t) * 10.0) as u32;
            (
                hash % 4,
                ((hash % 16 > 8) as u32) << 1,
                ((hash % 10 < 3) as u32),
            )
        };

        if x == 0 {
            blit_sub(
                self.sheet.bytes,
                dst.0,
                dst.1,
                10,
                10,
                60 + variant * 10,
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
                let state =
                    &self.grid[grid_from_vec(base_x + offset_x, base_y + offset_y) as usize];
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
                    }

                    CellState::House2(state, i) => {
                        let src_x = match state {
                            BuildingState::Solid => 0,
                            BuildingState::Burning => 20 + burning_x_offset,
                            BuildingState::Destroyed => 60,
                        };

                        self.draw_multi_grid_sprite(*i, 2, src_x, 60, dst_x, dst_y);
                    }

                    CellState::BigRock(i) => {
                        self.draw_multi_grid_sprite(*i, 2, 0, 40, dst_x, dst_y)
                    }
                    CellState::Rock => self.draw_grid_sprite(30, 40, dst_x, dst_y),
                    CellState::Lamppost(i) => {
                        self.draw_multi_grid_sprite(*i, 1, 20, 40, dst_x, dst_y)
                    }
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
                    }
                    CellState::Farm(i) => self.draw_multi_grid_sprite(*i, 2, 0, 110, dst_x, dst_y),
                    CellState::Hay(i) => self.draw_multi_grid_sprite(*i, 2, 40, 110, dst_x, dst_y),
                    _ => continue,
                }
            }
        }
    }

    // Draw the player cursors. Different colors assigned to each team
    unsafe fn draw_cursors(&self) {
        let index = self.current_player as usize;
        *DRAW_COLORS = if self.action_possible[index] { 0b0000_0000_0001_0000 } else { 0b0000_0000_0100_0000 };
        let (posx, posy) = vec_from_grid(self.cursors[index]);
        let posx = posx.saturating_sub(self.view_local_cameras[index].0) as i32;
        let posy = posy.saturating_sub(self.view_local_cameras[index].1) as i32;
        let offset: i32 = if self.tick > 30 && self.action_possible[index] { 1 } else { 0 };
        let mut offset_x: i32 = 0;
        let mut offset_y: i32 = 0;
        let mut offset_x2: i32 = 0;
        let mut offset_y2: i32 = 0;

        // this will need to be around a "selected element", i.e. a building. Rendering can be separated from logic, this means that cursor will do something idk
        match self.grid[self.cursors[index] as usize] {
            CellState::House(_, y)
            | CellState::House2(_, y)
            | CellState::BigRock(y)
            | CellState::Tree(y)
            | CellState::Stand(y) => {
                if y % 2 == 0 { offset_x2 = 10 } else { offset_x = -10 }
                if y > 1 { offset_y = -10 } else { offset_y2 = 10 }
            }

            CellState::Church(_x, y) => {
                if y % 2 == 0 { offset_x2 = 10 } else { offset_x = -10 }
                if y > 3 {
                    offset_y = -20
                } else if y > 1 {
                    offset_y = -10;
                    offset_y2 = 10
                } else {
                    offset_y2 = 20
                }
            }

            CellState::Lamppost(y) => {
                if y == 1 { offset_y = -10 } else { offset_y2 = 10 }
            }

            CellState::Farm(y) | CellState::Hay(y) => {
                if y % 2 == 0 { offset_x2 = 10 } else { offset_x = -10 }
            }

            _ => {}
        }

        self.draw_sprite(
            posx * 10 - offset + offset_x,
            posy * 10 - offset + offset_y,
            3,
            3,
            51,
            120,
        );
        let bottom_posy = posy * 10 + 7 + offset + offset_y2;
        if bottom_posy <= 120 {
            self.draw_sprite(
                posx * 10 + 7 + offset + offset_x2,
                bottom_posy,
                3,
                3,
                54,
                120,
            );
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

    // Draw a debug minimap
    unsafe fn debug_minimap(&self) {
        const MINIMAP_PIXEL_OFFSET_X: i32 = 128;
        const MINIMAP_PIXEL_OFFSET_Y: i32 = 123;

        fn flash_my_shit_twin(
            building_bigsprite_width: u8,
            big_sprite_subcell_index: u8,
            cursor_grid_pos: u16,
            tick: u8,
        ) -> Color {
            let (a, b) = calculate_big_sprite_root_position(building_bigsprite_width, big_sprite_subcell_index, cursor_grid_pos);
            let k = grid_from_vec(a, b);
            let hash = ((k as u64).wrapping_mul(95148)) ^ 0x856465;

            if (hash.wrapping_add(tick as u64) % 20) <= 10 {
                Color::Darker
            } else {
                Color::Darkest
            }
        }

        for x in 0..GRID_SIZE_X {
            for y in 0..GRID_SIZE_Y {
                let pos = grid_from_vec(x, y);
                let cell = self.grid[pos as usize];



                let color = match cell {
                    CellState::Empty => Color::Transparent,
                    CellState::IllagerClan(_, _) => Color::Darker,
                    CellState::VillagerClan(_) => Color::Darker,
                    CellState::House(BuildingState::Solid, _) | CellState::House2(BuildingState::Solid, _) | CellState::Church(BuildingState::Solid, _) => Color::Lighter,
                    CellState::House(BuildingState::Burning, j) => flash_my_shit_twin(2, j, pos, self.tick),
                    CellState::House2(BuildingState::Burning, j) => flash_my_shit_twin(2, j, pos, self.tick),
                    CellState::Church(BuildingState::Burning, j) => flash_my_shit_twin(2, j, pos, self.tick),
                    CellState::House(_, _) | CellState::House2(_, _) | CellState::Church(_, _) => Color::Darkest,
                    CellState::BigRock(_) | CellState::Rock | CellState::Lamppost(_) | CellState::Bell | CellState::Tree(_) | CellState::Stand(_) | CellState::Farm(_) | CellState::Hay(_) => Color::Lighter,
                };

                Self::set_rect_colors(color, color);
                rect(MINIMAP_PIXEL_OFFSET_X+x as i32, MINIMAP_PIXEL_OFFSET_Y+y as i32, 1, 1);
            }
        }
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

    let offset = match dir {
        Direction::N => (0, -1),
        Direction::E => (1, 0),
        Direction::S => (0, 1),
        Direction::W => (-1, 0),
        Direction::NE => (1, -1),
        Direction::SE => (1, 1),
        Direction::NW => (-1, -1),
        Direction::SW => (-1, 1),
    };

    x += offset.0;
    y += offset.1;

    let (x, y) = (x as u8, y as u8);
    ((0..GRID_SIZE_Y).contains(&y) && (0..GRID_SIZE_X).contains(&x)).then(|| grid_from_vec(x, y))
}

// calculate the bottom-left "root" position of the bigsprite given a cell state
fn calculate_big_sprite_root_position(
    building_bigsprite_width: u8,
    big_sprite_subcell_index: u8,
    cursor_grid_pos: u16,
) -> (u8, u8) {
    let j = big_sprite_subcell_index;
    let (a, b) = vec_from_grid(cursor_grid_pos);

    // calculate offset of the cursor *inside* of the bigsprite compared to the root position of the building
    // we then use this to calculate the bottom-left corner position of the bigsprite
    let cursor_offset_in_bigsprite_x = j % building_bigsprite_width;
    let cursor_offset_in_bigsprite_y = j / building_bigsprite_width; 

    // calculate the root position of the building
    let building_root_x = (a - cursor_offset_in_bigsprite_x) as i16;
    let building_root_y = (b - cursor_offset_in_bigsprite_y) as i16;

    return (building_root_x as u8, building_root_y as u8);
}

// picks a random location on the skirts of a building (on the outline)
// returns None if the building is completely surrounded
// returns Some with a position of a cell if it DID find a valid cell
fn pick_random_location_building_outline(
    board: &Board,
    building_bigsprite_width: u8,
    building_bigsprite_height: u8,
    big_sprite_subcell_index: u8,
    cursor_grid_pos: u16,
) -> Option<u16> {
    /* 
        pseudocode
        - make array
        - find all possible positions by figuring out position and working off of that
        - discard cells that are full / occupied by buildings
        - pick random cell in the range of that array
    */
    
    // calculate the root position of the building
    let (building_root_x, building_root_y) = calculate_big_sprite_root_position(building_bigsprite_width, big_sprite_subcell_index, cursor_grid_pos);
    let building_root_x = building_root_x as i16;
    let building_root_y = building_root_y as i16;

    // go over the outline cells of the building
    let mut possible_cells= Vec::<u16>::new();
    for x in (building_root_x - 1)..(building_root_x + building_bigsprite_width as i16 + 1)  {
        for y in (building_root_y - 1)..(building_root_y + building_bigsprite_height as i16 + 1)  {
            // make sure to discard cells that are outside the bounds of the map
            if x >= 0 && x < GRID_SIZE_X as i16 && y >= 0 && y < GRID_SIZE_Y as i16 {
                possible_cells.push(grid_from_vec(x as u8, y as u8))
            }
        }
    }

    // first step: get rid of cells that are occupied
    // this also discards cells that are *inside* the building themselves
    let possible_cells: Vec::<u16> = possible_cells.into_iter().filter(|position: &u16| matches!(board[*position as usize], CellState::Empty)).collect();

    // second step: pick a random cell if we can
    if possible_cells.len() == 0 {
        None
    } else {
        let rng = fastrand::usize(0..possible_cells.len());
        Some(possible_cells[rng])
    }
}

fn get_neighbours(index: u16) -> [u16; 8] {
    let y = GRID_SIZE_X as i16;
    let index = index as i16;
    let mut r = [u16::MAX; 8];

    for result in [
        index + y,
        index + 1 + y,
        index + 1,
        index + 1 - y,
        index - y,
        index - 1 - y,
        index - 1,
        index - 1 + y,
    ].iter().enumerate() {
        trace("hi".to_owned() + &result.0.to_string());
        let (x, y) = vec_from_grid(*result.1 as u16);
        if (0..GRID_SIZE_Y).contains(&y) && (0..GRID_SIZE_X).contains(&x) { r[result.0] = *result.1 as u16 } 
    }

    return r
}

/*
// I am going to gouge out my eyeballs
fn get_neighbours_in_range(index: u16, range_x: Range<i16>, range_y: Range<i16>) -> [u16; 8] {
    let index = vec_from_grid(index);
    let mut r: [u16; range_x * range_y] = [u16::MAX; 8];
    let mut vec_index = 0;
    for x in range_x.clone() {
        for y in range_y.clone() {
            r[vec_index] = if (0..GRID_SIZE_Y).contains(&((index.1 as i16 + y) as u8)) && (0..GRID_SIZE_X).contains(&((index.0 as i16 + x) as u8)) { grid_from_vec((index.0 as i16 + x) as u8, (index.1 as i16 + y) as u8) } else { u16::MAX };
            vec_index += 1;
        }
    }

    /*
    for result in [
        index + y,
        index + 1 + y,
        index + 1,
        index + 1 - y,
        index - y,
        index - 1 - y,
        index - 1,
        index - 1 + y,
    ].iter().enumerate() {
        trace("hi".to_owned() + &result.0.to_string());
        let (x, y) = vec_from_grid(*result.1 as u16);
        if (0..GRID_SIZE_Y).contains(&y) && (0..GRID_SIZE_X).contains(&x) { r[result.0] = *result.1 as u16 } 
    }
    */

    return r
}
*/

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
