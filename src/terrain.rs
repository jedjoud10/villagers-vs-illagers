use crate::{CellState, HouseState};

// Generate a grid with some interesting terrain
pub fn generate() -> [CellState; 192] {
    let mut grid = [CellState::Empty; 192];
    
    for _ in 0..3 {
        let x = fastrand::usize(0..175);
        make_house(x, &mut grid);
    }

    grid
}

fn make_house(index: usize, grid: &mut [CellState; 192]) {
    let valid = matches!(grid[index], CellState::Empty) && 
        matches!(grid[index + 1], CellState::Empty) && 
        matches!(grid[index + 16], CellState::Empty) && 
        matches!(grid[index + 17], CellState::Empty);

    if !valid {
        return;
    }

    grid[index] = CellState::House(HouseState::Solid, 0);
    grid[index + 1] = CellState::House(HouseState::Solid, 1);
    grid[index + 16] = CellState::House(HouseState::Solid, 2);
    grid[index + 17] = CellState::House(HouseState::Solid, 3);
}