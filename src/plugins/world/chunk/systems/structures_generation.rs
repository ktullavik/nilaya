use bevy::prelude::*;
use crate::plugins::world::WorldMap;
use crate::plugins::world::chunk::components::BlockType;
use crate::{CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_VOL};


pub fn add_cactus (height: usize, x: usize, y: usize, z: usize, blocks: &mut [BlockType; CHUNK_VOL]) {
    for i in 1..height {
        if y+i < CHUNK_HEIGHT-1 {
            blocks[x + (y+i)*CHUNK_WIDTH + z*CHUNK_WIDTH*CHUNK_HEIGHT] = BlockType::Cactus;
        }
    }
}


pub fn add_tree(
    height: usize,
    chunk_pos: (i32, i32),
    x: usize, y: usize, z: usize,
    world_map: &mut ResMut<WorldMap>,
    blocks: &mut [BlockType; CHUNK_VOL]) {

    blocks[x + (y+height)*CHUNK_WIDTH + z*CHUNK_WIDTH*CHUNK_HEIGHT] = BlockType::Leaves;

    let mut reserved_blocks_x = [BlockType::Air; CHUNK_VOL];
    let mut reserved_blocks_neg_x = [BlockType::Air; CHUNK_VOL];
    let mut reserved_blocks_z = [BlockType::Air; CHUNK_VOL];
    let mut reserved_blocks_neg_z = [BlockType::Air; CHUNK_VOL];

    if world_map.reserved_chunk_data.contains_key(&(chunk_pos.0 + 1, chunk_pos.1)) {
        reserved_blocks_x = world_map.reserved_chunk_data[&(chunk_pos.0 + 1, chunk_pos.1)];
    }
    if world_map.reserved_chunk_data.contains_key(&(chunk_pos.0 - 1, chunk_pos.1)) {
        reserved_blocks_neg_x = world_map.reserved_chunk_data[&(chunk_pos.0 - 1, chunk_pos.1)];
    }
    if world_map.reserved_chunk_data.contains_key(&(chunk_pos.0, chunk_pos.1 + 1)) {
        reserved_blocks_z = world_map.reserved_chunk_data[&(chunk_pos.0, chunk_pos.1 + 1)];
    }
    if world_map.reserved_chunk_data.contains_key(&(chunk_pos.0, chunk_pos.1 - 1)) {
        reserved_blocks_neg_z = world_map.reserved_chunk_data[&(chunk_pos.0, chunk_pos.1 - 1)];
    }

    let mut need_x = false;
    let mut need_neg_x = false;
    let mut need_z = false;
    let mut need_neg_z = false;

    for i in 1..height {
        if y+i < CHUNK_HEIGHT-1 {
            blocks[x + (y+i)*CHUNK_WIDTH + z*CHUNK_WIDTH*CHUNK_HEIGHT] = BlockType::WoodLog;
        }

        for j in 1..height-i-1 {

            if x+j >= CHUNK_WIDTH {
                let index = (x as i32 + j as i32-CHUNK_WIDTH as i32) as usize + (y+i+2) * CHUNK_WIDTH + z * CHUNK_HEIGHT * CHUNK_WIDTH;
                reserved_blocks_x[index] = BlockType::Leaves;
                need_x = true;
            }
            else {
                blocks[x+j + (y+i+2) * CHUNK_WIDTH + z * CHUNK_HEIGHT * CHUNK_WIDTH] = BlockType::Leaves;
            }

            if (x as i32 - j as i32) < 0 {
                let index = (x as i32 - j as i32 + CHUNK_WIDTH as i32) as usize + (y+i+2) * CHUNK_WIDTH + z * CHUNK_HEIGHT * CHUNK_WIDTH;
                reserved_blocks_neg_x[index] = BlockType::Leaves;
                need_neg_x = true;
            }
            else {
                blocks[x-j + (y+i+2) * CHUNK_WIDTH + z * CHUNK_HEIGHT * CHUNK_WIDTH] = BlockType::Leaves;
            }

            if z+j >= CHUNK_WIDTH {
                let index = x + (y+i+2) * CHUNK_WIDTH + (z as i32 + j as i32 - CHUNK_WIDTH as i32) as usize * CHUNK_HEIGHT * CHUNK_WIDTH;
                reserved_blocks_z[index] = BlockType::Leaves;
                need_z = true;
            }
            else {
                blocks[x + (y+i+2) * CHUNK_WIDTH + (z+j) * CHUNK_HEIGHT * CHUNK_WIDTH] = BlockType::Leaves;
            }

            if (z as i32 - j as i32) < 0 {
                let index = x + (y+i+2) * CHUNK_WIDTH + (z as i32 - j as i32+CHUNK_WIDTH as i32) as usize * CHUNK_HEIGHT * CHUNK_WIDTH;
                reserved_blocks_neg_z[index] = BlockType::Leaves;
                need_neg_z = true;
            }
            else {
                blocks[x + (y+i+2) * CHUNK_WIDTH + (z-j) * CHUNK_HEIGHT * CHUNK_WIDTH] = BlockType::Leaves;
            }
        }
    }

    if need_x {world_map.reserved_chunk_data.insert((chunk_pos.0 + 1, chunk_pos.1), reserved_blocks_x);}
    if need_neg_x {world_map.reserved_chunk_data.insert((chunk_pos.0 - 1, chunk_pos.1), reserved_blocks_neg_x);}
    if need_z {world_map.reserved_chunk_data.insert((chunk_pos.0, chunk_pos.1 + 1), reserved_blocks_z);}
    if need_neg_z {world_map.reserved_chunk_data.insert((chunk_pos.0, chunk_pos.1 - 1), reserved_blocks_neg_z);}
}