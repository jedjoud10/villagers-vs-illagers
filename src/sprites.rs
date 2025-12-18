use crate::*;

// Sprite that contains bit-encoded data (either 1BPP or 2BPP) and corresponding width/height
pub struct Sprite {
    pub bytes: &'static [u8],
    pub width: u32,
    pub height: u32,
    pub flags: u32,
}

#[macro_export]
macro_rules! sprite {
    ($file:expr $(,)?) => {{
        let data: &'static [u8] = include_bytes!($file);
        let width = data[0] as u32;
        let height = data[1] as u32;
        let flags = data[2] as u32;
        let bytes = data.split_at(3);
        Sprite {
            bytes: bytes.1,
            width,
            height,
            flags,
        }
    }};
}

const SHEET: Sprite =  sprite!("../packed/sprite.pak");

pub fn draw_villager_entity(dst_x: i32, dst_y: i32, _type: &VillagerClan) {
    fn custom_rectangular_sublice(dst_x: i32, dst_y: i32, width: u32, height: u32, src_x: u32, src_y: u32, inside_rect_offset_x: u32, inside_rect_offset_y: u32) {
        draw_sprite(dst_x + inside_rect_offset_x as i32, dst_y + inside_rect_offset_y as i32, width, height, src_x + inside_rect_offset_x, src_y + inside_rect_offset_y);
    }

    match _type {
        VillagerClan::Villager => {
            custom_rectangular_sublice(dst_x, dst_y, 6, 8, 40, 0, 2, 0); // top
            custom_rectangular_sublice(dst_x, dst_y, 4, 2, 40, 0, 3, 8); // bottom
        },
        VillagerClan::Smith(_) => {
            custom_rectangular_sublice(dst_x, dst_y, 6, 8, 60, 0, 2, 0); // top
            custom_rectangular_sublice(dst_x, dst_y, 4, 2, 60, 0, 3, 8); // bottom
        },
        VillagerClan::Farmer => {
            custom_rectangular_sublice(dst_x, dst_y, 6, 1, 50, 0, 2, 0); // hat, top
            custom_rectangular_sublice(dst_x, dst_y, 10, 1, 50, 0, 0, 1); // hat, bottom

            custom_rectangular_sublice(dst_x, dst_y, 6, 6, 50, 0, 2, 2); // top
            custom_rectangular_sublice(dst_x, dst_y, 4, 2, 50, 0, 3, 8); // bottom
        },
        VillagerClan::Golem(_, GolemState::Idle) => {
            custom_rectangular_sublice(dst_x, dst_y, 2, 3, 70, 0, 0, 6); // left most vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 10, 70, 0, 2, 0); // left vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 9, 70, 0, 4, 0); // center vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 10, 70, 0, 6, 0); // right vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 3, 70, 0, 8, 6); // right most vertical strip
        },
        VillagerClan::Golem(_, GolemState::Attack) => {
            custom_rectangular_sublice(dst_x, dst_y, 2, 3, 60, 10, 0, 4); // left most vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 10, 60, 10, 2, 0); // left vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 9, 60, 10, 4, 0); // center vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 10, 60, 10, 6, 0); // right vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 3, 60, 10, 8, 4); // right most vertical strip
        },
        VillagerClan::Golem(_, GolemState::Broken) => {
            custom_rectangular_sublice(dst_x, dst_y, 2, 3, 70, 10, 0, 6); // left most vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 10, 70, 10, 2, 0); // left vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 9, 70, 10, 4, 0); // center vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 10, 70, 10, 6, 0); // right vertical strip
            custom_rectangular_sublice(dst_x, dst_y, 2, 3, 70, 10, 8, 6); // right most vertical strip
        }
        _ => {}
    };
    
    /*
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
    draw_grid_sprite(src_x, src_y, dst_x, dst_y);
    */
}

pub fn draw_illager_entity(dst_x: i32, dst_y: i32, _type: &IllagerClan, state: &IllagerState) {
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
    draw_grid_sprite(src_x, src_y, dst_x, dst_y)
}

// Common functionality for rendering multi-sprite buildings (houses, church, bell, torch pole)
// mega_width are in bigsprite size (so for house this would be 2)
pub fn draw_multi_grid_sprite(
    index: u8,
    mega_width: u8,
    src_x: u32,
    src_y: u32,
    dst_x: i32,
    dst_y: i32,
) {
    let x_offset = (index % mega_width) as u32;
    let y_offset = (index / mega_width) as u32;
    draw_grid_sprite(x_offset * (CELL_SIZE as u32) + src_x, y_offset * (CELL_SIZE as u32) + src_y, dst_x, dst_y)
}

// Util function for grid sprites only
pub fn draw_grid_sprite(src_x: u32, src_y: u32, dst_x: i32, dst_y: i32) {
    draw_sprite(dst_x, dst_y, (CELL_SIZE as u32), (CELL_SIZE as u32), src_x, src_y)
}

// Draw a general sprite from the main sprite sheet
pub fn draw_sprite(x: i32, y: i32, width: u32, height: u32, src_x: u32, src_y: u32) {
    blit_sub(
        SHEET.bytes,
        x,
        y,
        width,
        height,
        src_x,
        src_y,
        SHEET.width,
        SHEET.flags,
    );
}

pub fn draw_sprite_with_extra_flags(x: i32, y: i32, width: u32, height: u32, src_x: u32, src_y: u32, flags: u32) {
    blit_sub(
        SHEET.bytes,
        x,
        y,
        width,
        height,
        src_x,
        src_y,
        SHEET.width,
        SHEET.flags | flags,
    );
}