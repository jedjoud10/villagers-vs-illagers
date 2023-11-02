use crate::{CellState, BuildingState};

// Terrain feature that we could generate
struct Feature {
    closure: fn(u8) -> CellState,
    dimensions: (u8, u8),
    max_attempts_to_spawn: u8,
}

// Generate a grid with some interesting terrain
pub fn generate() -> [CellState; 192] {
    let mut grid = [CellState::Empty; 192];

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
            closure: |i| CellState::Lampost(i),
            dimensions: (1, 2),
            max_attempts_to_spawn: 3,
        }
    ];

    for Feature { closure, dimensions, max_attempts_to_spawn } in features {
        for _ in 0..max_attempts_to_spawn {
            let x = fastrand::usize(0..192);
            spawn_building(x, &mut grid, dimensions.0, dimensions.1, closure);
        }
    }

    grid
}

// Spawn any arbitrary building using a lambda closure
fn spawn_building(index: usize, grid: &mut [CellState; 192], width: u8, height: u8, function: impl Fn(u8) -> CellState) {
    // Makes sure we won't have a building that "extends" into the map border
    let (x, y) = crate::vec_from_grid(index as u8);
    if x >= (16u8 - width) || y >= (12u8 - height) {
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
        index as u8 + a
    });

    // Can't use Vec. Womp womp
    let mut cache = [0u8; 2*3];
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