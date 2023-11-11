use crate::{CellState, BuildingState, AREA, GRID_SIZE_X, GRID_SIZE_Y, rng::Rng};

// Terrain feature that we could generate
struct Feature {
    closure: fn(u8) -> CellState,
    dimensions: (u8, u8),
    max_attempts_to_spawn: u8,
}

// Generate a grid with some interesting terrain
pub fn generate(rng: &mut Rng) -> Box<[CellState; AREA]> {
    let temp = vec![CellState::Empty; AREA].into_boxed_slice();
    let mut grid: Box<[CellState; AREA]> = unsafe { Box::from_raw(Box::into_raw(temp) as *mut [CellState; AREA]) };

    let features = [
        Feature {
            closure: |i| CellState::House(BuildingState::Solid, i),
            dimensions: (2, 2),
            max_attempts_to_spawn: 3,
        },

        Feature {
            closure: |i| CellState::Church(BuildingState::Solid, i),
            dimensions: (2, 3),
            max_attempts_to_spawn: 3,
        },

        Feature {
            closure: |i| CellState::BigRock(i),
            dimensions: (2, 2),
            max_attempts_to_spawn: 3,
        },

        Feature {
            closure: |i| CellState::Tree(i),
            dimensions: (2, 2),
            max_attempts_to_spawn: 7,
        },

        Feature {
            closure: |_| CellState::Rock,
            dimensions: (1, 1),
            max_attempts_to_spawn: 4,
        },

        Feature {
            closure: |_| CellState::Bell,
            dimensions: (1, 1),
            max_attempts_to_spawn: 2,
        },

        Feature {
            closure: |i| CellState::Hay(i),
            dimensions: (2, 1),
            max_attempts_to_spawn: 3,
        },

        Feature {
            closure: |i| CellState::Flowers(i),
            dimensions: (2, 1),
            max_attempts_to_spawn: 3,
        },

        Feature {
            closure: |i| CellState::Lamppost(i),
            dimensions: (1, 2),
            max_attempts_to_spawn: 3,
        }
    ];

    for Feature { closure, dimensions, max_attempts_to_spawn } in features {
        for _ in 0..max_attempts_to_spawn {
            let x = rng.u16(0..(AREA as u16));
            spawn_building(x, grid.as_mut_slice(), dimensions.0, dimensions.1, closure);
        }
    }

    grid
}

// Maximum number of cells for one building
const MAX_BUILDING_COUNT_CACHE: usize = 2*3;

// Spawn any arbitrary building using a lambda closure
fn spawn_building(index: u16, grid: &mut [CellState], width: u8, height: u8, function: impl Fn(u8) -> CellState) {
    // Makes sure we won't have a building that "extends" into the map border
    let (x, y) = crate::vec_from_grid(index);

    if x >= (GRID_SIZE_X - width) || y >= (GRID_SIZE_Y - height) {
        return
    }

    // Calculates indices for grid cells that will be modified
    let indices = (0..(width * height)).into_iter().map(|i| {
        // Convert "i" variant number of building into local space (on a building scale)
        let (ax, ay) = {
            let index = i as u8;
            let x = index % width;
            let y = index / width;
            (x, y)
        };

        // Convert it back to grid space (on the map scale though)
        let a = crate::grid_from_vec(ax, ay);
        index as u16 + a
    });

    // Can't use Vec. Womp womp
    let mut cache = [0u16; MAX_BUILDING_COUNT_CACHE];
    let mut count = 0;
    for (src, dst) in indices.zip(cache.iter_mut()) {
        *dst = src;
        count += 1;
    }

    // Make sure all the cells are empty, otherwise don't do anything
    if !cache[..count].iter().all(|i| {
        matches!(grid[*i as usize], CellState::Empty)
    }) {
        return;
    }

    // Set the proper cells
    for (variant, &i) in cache[..count].into_iter().enumerate() {
        grid[i as usize] = function(variant as u8);
    }
}