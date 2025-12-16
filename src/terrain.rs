use crate::{grid_from_vec, BuildingState, CellState, AREA, GRID_SIZE_X, GRID_SIZE_Y};

const MAX_SPAWN_ATTEMPTS: usize = 1000;

// Terrain feature that we could generate
struct Feature {
    // closure for custom building / bigsprite spawning logic
    // called for each x,y in the dimensions of the feature
    closure: fn(u8) -> CellState,

    // calculates if we should spawn building given a random position input 
    probability: fn(u8, u8) -> bool,

    // dimensions of the feature in sprite size
    dimensions: (u8, u8),

    // minimum..maximum count of the number of spawns that we should do
    spawn_min_max: (u16, u16),

    // simple AABB that depicts the spawning bounds
    range_to_spawn: [(u8, u8); 2],
}

fn dist(x: i8, y: i8, center_x: i8, center_y: i8) -> u8 {
    // variant that makes it a diagonal (manhattan distance)
    //((x - center_x).abs() + (y - center_y).abs()) as u8

    // variant that makes it a perfect square (chebyshev distance)
    (x - center_x).abs().max((y - center_y).abs()) as u8 * 2
}

// Generate a grid with some interesting terrain
pub fn generate() -> Box<[CellState; AREA]> {
    let temp: Box<[CellState]> = vec![CellState::Empty; AREA].into_boxed_slice();
    let mut grid: Box<[CellState; AREA]> =
        unsafe { Box::from_raw(Box::into_raw(temp) as *mut [CellState; AREA]) };

    // NOTE: features defined EARLIER in this array will be PRIORITIZED
    // i.e: we will try spawning the houses FIRST, *then* we will try spawning tree
    let features = [
        Feature {
            closure: |i| CellState::House(BuildingState::Solid, i),
            probability: |x, y| {
                if x % 3 < 2 {
                    return false;
                }

                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) < 4
            },
            dimensions: (2, 2),
            spawn_min_max: (3, 6),
            range_to_spawn: [(10, 10), (20, 20)],
        },
        Feature {
            closure: |i| CellState::House2(BuildingState::Solid, i),
            probability: |x, y| {
                if x % 3 < 2 {
                    return false;
                }

                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) < 16
            },
            dimensions: (2, 2),
            spawn_min_max: (1, 4),
            range_to_spawn: [(10, 10), (20, 20)],
        },
        Feature {
            closure: |i| CellState::Church(BuildingState::Solid, i),
            probability: |x, y| {
                if x % 3 < 2 {
                    return false;
                }

                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) < 16
            },
            dimensions: (2, 3),
            spawn_min_max: (2, 6),
            range_to_spawn: [(10, 10), (20, 20)],
        },
        Feature {
            closure: CellState::BigRock,
            probability: |x, y| {
                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) > 15
            },
            dimensions: (2, 2),
            spawn_min_max: (10, 60),
            range_to_spawn: [(0, 0), (GRID_SIZE_X, GRID_SIZE_Y)],
        },
        Feature {
            closure: CellState::Tree,
            probability: |x, y| {
                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) > 15
            },
            dimensions: (2, 2),
            spawn_min_max: (30, 60),
            range_to_spawn: [(0, 0), (GRID_SIZE_X, GRID_SIZE_Y)],
        },
        Feature {
            closure: |_| CellState::Rock,
            probability: |x, y| {
                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) > 15
            },
            dimensions: (1, 1),
            spawn_min_max: (20, 60),
            range_to_spawn: [(0, 0), (GRID_SIZE_X, GRID_SIZE_Y)],
        },
        Feature {
            closure: |_| CellState::Bell,
            probability: |x, y| {
                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) < 15
            },
            dimensions: (1, 1),
            spawn_min_max: (1, 8),
            range_to_spawn: [(13, 13), (17, 17)],
        },
        Feature {
            closure: CellState::Hay,
            probability: |x, y| {
                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) < 15
            },
            dimensions: (2, 1),
            spawn_min_max: (1, 8),
            range_to_spawn: [(10, 10), (20, 20)],
        },
        Feature {
            closure: CellState::Farm,
            probability: |x, y| {
                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) < 15
            },
            dimensions: (2, 1),
            spawn_min_max: (3, 8),
            range_to_spawn: [(8, 8), (22, 22)],
        },
        Feature {
            closure: CellState::Lamppost,
            probability: |x, y| {
                let (x, y) = (x as i8, y as i8);
                dist(x, y, 15, 15) < 15
            },
            dimensions: (1, 2),
            spawn_min_max: (2, 8),
            range_to_spawn: [(12, 12), (18, 18)],
        },
    ];

    for Feature {
        closure,
        probability,
        range_to_spawn,
        dimensions,
        spawn_min_max,
    } in features
    {
        let mut count = 0;
        let min_count = spawn_min_max.0;
        let max_count = spawn_min_max.1;

        'a: for _ in 0..(MAX_SPAWN_ATTEMPTS) {
            let x = fastrand::u8(range_to_spawn[0].0..(range_to_spawn[1].0));
            let y = fastrand::u8(range_to_spawn[0].1..(range_to_spawn[1].1));

            if !probability(x, y) {
                continue;
            }

            let index = grid_from_vec(x, y);
            if spawn_building(
                index,
                grid.as_mut_slice(),
                dimensions.0,
                dimensions.1,
                closure,
            ) {
                count += 1;
            }

            // we can no longer spawn any more features of this type
            if count > max_count {
                break 'a;
            }

            // we have spawned *enough*, but we could spawn more if we wanted to (50/50 random chance to continue)
            if count > min_count && fastrand::bool() {
                break 'a;
            }
        }
    }

    grid
}

// Maximum number of cells for one building
const MAX_BUILDING_COUNT_CACHE: usize = 2 * 3;

// Spawn any arbitrary building using a lambda closure
fn spawn_building(
    index: u16,
    grid: &mut [CellState],
    width: u8,
    height: u8,
    function: impl Fn(u8) -> CellState,
) -> bool {
    // Makes sure we won't have a building that "extends" into the map border
    let (x, y) = crate::vec_from_grid(index);

    if x > (GRID_SIZE_X - width) || y > (GRID_SIZE_Y - height) {
        return false;
    }

    // Calculates indices for grid cells that will be modified
    let indices = (0..(width * height)).map(|i| {
        // Convert "i" variant number of building into local space (on a building scale)
        let (ax, ay) = {
            let index = i;
            let x = index % width;
            let y = index / width;
            (x, y)
        };

        // Convert it back to grid space (on the map scale though)
        let a = crate::grid_from_vec(ax, ay);
        index + a
    });

    // Can't use Vec. Womp womp
    let mut cache = [0u16; MAX_BUILDING_COUNT_CACHE];
    let mut count = 0;
    for (src, dst) in indices.zip(cache.iter_mut()) {
        *dst = src;
        count += 1;
    }

    // Make sure all the cells are empty, otherwise don't do anything
    if !cache[..count]
        .iter()
        .all(|i| matches!(grid[*i as usize], CellState::Empty))
    {
        return false;
    }

    // Set the proper cells
    for (variant, &i) in cache[..count].iter().enumerate() {
        grid[i as usize] = function(variant as u8);
    }

    true
}
