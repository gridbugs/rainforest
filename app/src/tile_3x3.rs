use crate::{
    colour,
    fields::{GroundField, LogField, TeaField},
};
use chargrid::core::prelude::*;
use grid_2d::coord_2d::{Axis, Coord, Size};
use rainforest_game::{EntityTile, Game, Tile, VisibilityCell};
use rgb_int::Rgb24;

pub fn render_3x3_from_visibility(
    screen_coord: Coord,
    world_coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    ground_field: &GroundField,
    log_field: &LogField,
    tea_field: &TeaField,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    let ctx = ctx.add_offset(screen_coord * 3);
    let mut render_tile = |entity, tile, ctx| match tile {
        Tile::Wall => {
            let below = world_coord + Coord::new(0, 1);
            if game.contains_wall(below) {
                wall_top(world_coord, log_field, ctx, fb);
            } else {
                wall_front(world_coord, log_field, ctx, fb);
            }
        }
        Tile::Floor => floor(ctx, fb),
        Tile::Ground => {
            ground_field
                .get(world_coord)
                .map(|rgb24| ground(rgb24, ctx, fb));
        }
        Tile::Player => player(ctx, fb),
        Tile::Tree0 => tree0(ctx, fb),
        Tile::Tree1 => tree1(ctx, fb),
        Tile::Tree2 => tree2(ctx, fb),
        Tile::Window(Axis::Y) => {
            window_y(world_coord, log_field, ctx, fb);
        }
        Tile::Window(Axis::X) => window_x(world_coord, log_field, ctx, fb),
        Tile::DoorOpen(Axis::X) => door_open_x(ctx, fb),
        Tile::DoorOpen(Axis::Y) => door_open_y(ctx, fb),
        Tile::DoorClosed(Axis::X) => door_closed_x(ctx, fb),
        Tile::DoorClosed(Axis::Y) => door_closed_y(ctx, fb),
        Tile::Water => {
            let colour_hint = game.colour_hint(entity).unwrap();
            water(colour_hint, ctx, fb);
        }
        Tile::RuinsWall => {
            let below = world_coord + Coord::new(0, 1);
            if game.contains_wall(below) {
                ruins_wall_top(ctx, fb);
            } else {
                ruins_wall_front(ctx, fb);
            }
        }
        Tile::RuinsFloor => ruins_floor(ctx, fb),
        Tile::Altar => altar(ctx, fb),
        Tile::BulletinBoard => bulletin_board(ctx, fb),
        Tile::Lamp => lamp(ctx, fb),
        Tile::PierFloor => pier_floor(ctx, fb),
        Tile::Grass => grass(ctx, fb),
        Tile::FlatGrass => flat_grass(ctx, fb),
        Tile::Rock => rock(ctx, fb),
        Tile::Flower => flower(ctx, fb),
        Tile::Bed => bed(ctx, fb),
        Tile::ChairLeftFacing => chair_left_facing(ctx, fb),
        Tile::ChairRightFacing => chair_right_facing(ctx, fb),
        Tile::Teapot => teapot(ctx, fb),
        Tile::Tea => tea(tea_field.get(world_coord).unwrap(), ctx, fb),
    };
    let tile_layers = visibility_cell.tile_layers();
    if let Some(EntityTile { entity, tile }) = tile_layers.floor {
        render_tile(entity, tile, ctx.add_depth(0));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.feature {
        render_tile(entity, tile, ctx.add_depth(1));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.item {
        render_tile(entity, tile, ctx.add_depth(2));
    }
    if let Some(EntityTile { entity, tile }) = tile_layers.character {
        render_tile(entity, tile, ctx.add_depth(3));
    }
}

pub fn render_3x3_from_visibility_remembered(
    screen_coord: Coord,
    world_coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    log_field: &LogField,
    tea_field: &TeaField,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    let ctx = ctx.add_offset(screen_coord * 3);
    let mut render_tile = |tile, ctx| match tile {
        Tile::Wall => {
            let below = world_coord + Coord::new(0, 1);
            if game.contains_wall(below) {
                wall_top(world_coord, log_field, ctx, fb);
            } else {
                wall_front(world_coord, log_field, ctx, fb);
            }
        }
        Tile::Floor => floor(ctx, fb),
        Tile::Ground => ground(Rgb24::new_grey(10), ctx, fb),
        Tile::Tree0 => tree0(ctx, fb),
        Tile::Tree1 => tree1(ctx, fb),
        Tile::Tree2 => tree2(ctx, fb),
        Tile::Player => player(ctx, fb),
        Tile::Window(Axis::Y) => {
            window_y(world_coord, log_field, ctx, fb);
        }
        Tile::Window(Axis::X) => window_x(world_coord, log_field, ctx, fb),
        Tile::DoorOpen(Axis::X) => door_open_x(ctx, fb),
        Tile::DoorOpen(Axis::Y) => door_open_y(ctx, fb),
        Tile::DoorClosed(Axis::X) => door_closed_x(ctx, fb),
        Tile::DoorClosed(Axis::Y) => door_closed_y(ctx, fb),
        Tile::Water => water(Rgb24::new_grey(128), ctx, fb),
        Tile::RuinsWall => {
            let below = world_coord + Coord::new(0, 1);
            if game.contains_wall(below) {
                ruins_wall_top(ctx, fb);
            } else {
                ruins_wall_front(ctx, fb);
            }
        }
        Tile::RuinsFloor => ruins_floor(ctx, fb),
        Tile::Altar => altar(ctx, fb),
        Tile::BulletinBoard => bulletin_board(ctx, fb),
        Tile::Lamp => lamp(ctx, fb),
        Tile::PierFloor => pier_floor(ctx, fb),
        Tile::Grass => grass(ctx, fb),
        Tile::FlatGrass => flat_grass(ctx, fb),
        Tile::Rock => rock(ctx, fb),
        Tile::Flower => flower(ctx, fb),
        Tile::Bed => bed(ctx, fb),
        Tile::ChairLeftFacing => chair_left_facing(ctx, fb),
        Tile::ChairRightFacing => chair_right_facing(ctx, fb),
        Tile::Teapot => teapot(ctx, fb),
        Tile::Tea => tea(tea_field.get(world_coord).unwrap(), ctx, fb),
    };
    let tile_layers = visibility_cell.tile_layers();
    if let Some(EntityTile { entity: _, tile }) = tile_layers.floor {
        render_tile(tile, ctx.add_depth(0));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.feature {
        render_tile(tile, ctx.add_depth(1));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.item {
        render_tile(tile, ctx.add_depth(2));
    }
    if let Some(EntityTile { entity: _, tile }) = tile_layers.character {
        render_tile(tile, ctx.add_depth(3));
    }
}

fn floor(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::FLOOR_BACKGROUND),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▪')
            .with_foreground(colour::FLOOR_FOREGROUND),
    );
}

fn ground(foreground: Rgb24, ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::GROUND_BACKGROUND),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▪')
            .with_foreground(foreground.to_rgba32(255)),
    );
}

fn player(ctx: Ctx, fb: &mut FrameBuffer) {
    let bold = false;
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colour::PLAYER)
            .with_bold(bold),
    );
}

fn window_y(world_coord: Coord, log_field: &LogField, ctx: Ctx, fb: &mut FrameBuffer) {
    let field_base = Coord {
        x: world_coord.x * 3,
        y: world_coord.y * 6,
    };
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        let field_coord = field_base
            + Coord {
                x: offset.x,
                y: offset.y * 2,
            };
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character('▄')
                .with_background(
                    log_field
                        .get_horizontal(field_coord)
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                )
                .with_foreground(
                    log_field
                        .get_horizontal(field_coord + Coord::new(0, 1))
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                ),
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        let field_coord = field_base
            + Coord::new(0, 2)
            + Coord {
                x: offset.x,
                y: offset.y * 2,
            };
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_background(colour::WALL_FRONT)
                .with_background(
                    log_field
                        .get_horizontal(field_coord)
                        .unwrap()
                        .to_rgba32(255),
                )
                .with_foreground(
                    log_field
                        .get_horizontal(field_coord + Coord::new(0, 1))
                        .unwrap()
                        .to_rgba32(255),
                ),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        1,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default().with_character('▝').with_foreground(
            log_field
                .get_horizontal(field_base + Coord::new(0, 4))
                .unwrap()
                .saturating_scalar_mul_div(1, 2)
                .to_rgba32(255),
        ),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default().with_character('▀').with_foreground(
            log_field
                .get_horizontal(field_base + Coord::new(1, 4))
                .unwrap()
                .saturating_scalar_mul_div(1, 2)
                .to_rgba32(255),
        ),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default().with_character('▘').with_foreground(
            log_field
                .get_horizontal(field_base + Coord::new(2, 4))
                .unwrap()
                .saturating_scalar_mul_div(1, 2)
                .to_rgba32(255),
        ),
    );
}

fn window_x(world_coord: Coord, log_field: &LogField, ctx: Ctx, fb: &mut FrameBuffer) {
    let field_base = Coord {
        x: world_coord.x * 6,
        y: world_coord.y * 3,
    };
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        let field_coord = field_base
            + Coord {
                x: offset.x * 2,
                y: offset.y,
            };
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(
                    log_field
                        .get_vertical(field_coord)
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                )
                .with_foreground(
                    log_field
                        .get_vertical(field_coord + Coord::new(1, 0))
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                ),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::WINDOWS),
    );
    let field_base = Coord {
        x: world_coord.x * 3,
        y: world_coord.y * 6,
    };
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default().with_character('▝').with_foreground(
            log_field
                .get_horizontal(field_base + Coord::new(0, 4))
                .unwrap()
                .to_rgba32(255),
        ),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default().with_character('▀').with_foreground(
            log_field
                .get_horizontal(field_base + Coord::new(1, 4))
                .unwrap()
                .to_rgba32(255),
        ),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default().with_character('▘').with_foreground(
            log_field
                .get_horizontal(field_base + Coord::new(2, 4))
                .unwrap()
                .to_rgba32(255),
        ),
    );
}

fn door_closed_y(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colour::DOOR_BORDER)
            .with_background(colour::DOOR),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::DOOR)
            .with_background(colour::DOOR_BORDER),
    );
}

fn door_closed_x(ctx: Ctx, fb: &mut FrameBuffer) {
    door_closed_y(ctx, fb);
}

fn door_open_y(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::DOOR_BORDER),
    );
}

fn door_open_x(ctx: Ctx, fb: &mut FrameBuffer) {
    door_open_y(ctx, fb);
}

pub fn wall_top(world_coord: Coord, log_field: &LogField, ctx: Ctx, fb: &mut FrameBuffer) {
    let field_base = Coord {
        x: world_coord.x * 6,
        y: world_coord.y * 3,
    };
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        let field_coord = field_base
            + Coord {
                x: offset.x * 2,
                y: offset.y,
            };
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(
                    log_field
                        .get_vertical(field_coord)
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                )
                .with_foreground(
                    log_field
                        .get_vertical(field_coord + Coord::new(1, 0))
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                ),
        );
    }
}

pub fn wall_front(world_coord: Coord, log_field: &LogField, ctx: Ctx, fb: &mut FrameBuffer) {
    let field_base = Coord {
        x: world_coord.x * 3,
        y: world_coord.y * 6,
    };
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        let field_coord = field_base
            + Coord {
                x: offset.x,
                y: offset.y * 2,
            };
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character('▄')
                .with_background(
                    log_field
                        .get_horizontal(field_coord)
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                )
                .with_foreground(
                    log_field
                        .get_horizontal(field_coord + Coord::new(0, 1))
                        .unwrap()
                        .saturating_scalar_mul_div(1, 2)
                        .to_rgba32(255),
                ),
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        let field_coord = field_base
            + Coord::new(0, 2)
            + Coord {
                x: offset.x,
                y: offset.y * 2,
            };
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_background(colour::WALL_FRONT)
                .with_background(
                    log_field
                        .get_horizontal(field_coord)
                        .unwrap()
                        .to_rgba32(255),
                )
                .with_foreground(
                    log_field
                        .get_horizontal(field_coord + Coord::new(0, 1))
                        .unwrap()
                        .to_rgba32(255),
                ),
        );
    }
}

const TREE_DEPTH: i8 = 5;
pub fn tree0(ctx: Ctx, fb: &mut FrameBuffer) {
    let ctx = ctx.add_y(-4).add_depth(2);
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 3 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 4 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 5 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 3 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
}

pub fn tree1(ctx: Ctx, fb: &mut FrameBuffer) {
    let ctx = ctx.add_y(-4).add_depth(2);
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 3 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 4 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 5 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 3 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
}

pub fn tree2(ctx: Ctx, fb: &mut FrameBuffer) {
    let ctx = ctx.add_y(-4).add_depth(2);
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 3 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 4 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 5 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 4 },
        TREE_DEPTH,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD)
            .with_background(colour::LEAF),
    );
}

pub fn water(colour_hint: Rgb24, ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character('~')
                .with_foreground(colour_hint.saturating_scalar_mul_div(1, 1).to_rgba32(255))
                .with_background(colour_hint.saturating_scalar_mul_div(1, 3).to_rgba32(255)),
        );
    }
}

pub fn ruins_wall_top(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::RUINS_WALL_TOP),
        );
    }
}

pub fn ruins_wall_front(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::RUINS_WALL_TOP),
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord::new(0, 1),
            0,
            RenderCell::default()
                .with_character('░')
                .with_background(colour::RUINS_WALL_FRONT_BACKGROUND)
                .with_foreground(colour::RUINS_WALL_FRONT_FOREGROUND),
        );
    }
}

fn ruins_floor(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::RUINS_FLOOR_BACKGROUND),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▪')
            .with_foreground(colour::RUINS_FLOOR_FOREGROUND),
    );
}

pub fn altar(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 1),
        0,
        RenderCell::default()
            .with_character('║')
            .with_foreground(colour::ALTAR_TOP_FOREGROUND)
            .with_background(colour::RUINS_WALL_TOP),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('@')
            .with_foreground(colour::ALTAR_TOP_FOREGROUND)
            .with_background(colour::RUINS_WALL_TOP),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 1),
        0,
        RenderCell::default()
            .with_character('║')
            .with_foreground(colour::ALTAR_TOP_FOREGROUND)
            .with_background(colour::RUINS_WALL_TOP),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 2),
        0,
        RenderCell::default()
            .with_character('╠')
            .with_foreground(colour::ALTAR_FOREGROUND)
            .with_background(colour::RUINS_WALL_FRONT_BACKGROUND),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('╦')
            .with_foreground(colour::ALTAR_FOREGROUND)
            .with_background(colour::RUINS_WALL_FRONT_BACKGROUND),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 2),
        0,
        RenderCell::default()
            .with_character('╣')
            .with_foreground(colour::ALTAR_FOREGROUND)
            .with_background(colour::RUINS_WALL_FRONT_BACKGROUND),
    );
}

fn bulletin_board(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 0),
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 0),
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 0),
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 1),
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('≡')
            .with_background(colour::WOOD)
            .with_foreground(colour::BULLETIN_TEXT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 1),
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::WOOD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('│')
            .with_bold(true)
            .with_foreground(colour::WOOD),
    );
}

fn lamp(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 0),
        0,
        RenderCell::default()
            .with_character('▼')
            .with_bold(true)
            .with_foreground(colour::LAMP_LIGHT)
            .with_background(colour::LAMP_BASE),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('┼')
            .with_bold(true)
            .with_foreground(colour::LAMP_BASE),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('│')
            .with_bold(true)
            .with_foreground(colour::LAMP_BASE),
    );
}

fn pier_floor(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::PIER_FLOOR_BACKGROUND),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('▪')
            .with_foreground(colour::PIER_FLOOR_FOREGROUND),
    );
}

fn grass(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord::new(0, 1),
            3,
            RenderCell::default()
                .with_character('║')
                .with_bold(true)
                .with_foreground(colour::GRASS),
        );
    }
}

fn flat_grass(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord::new(0, 2),
            3,
            RenderCell::default()
                .with_character('║')
                .with_foreground(colour::GRASS),
        );
    }
}

fn rock(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character(' ')
            .with_background(colour::ROCK),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 0),
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::ROCK),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colour::ROCK),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 1),
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::ROCK),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 1),
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::ROCK),
    );
}

fn flower(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('├')
            .with_bold(true)
            .with_foreground(colour::GRASS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('▼')
            .with_bold(true)
            .with_foreground(colour::FLOWER),
    );
}

fn bed(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 1),
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::BED_MATRESS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::BED_MATRESS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 1),
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colour::FLOOR_BACKGROUND)
            .with_background(colour::BED_HEAD),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 2),
        0,
        RenderCell::default()
            .with_character('║')
            .with_bold(true)
            .with_foreground(colour::BED_LEGS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 2),
        0,
        RenderCell::default()
            .with_character('║')
            .with_bold(true)
            .with_foreground(colour::BED_LEGS),
    );
}

fn chair_left_facing(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 2),
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::CHAIR),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::CHAIR),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('▐')
            .with_foreground(colour::CHAIR),
    );
}

fn chair_right_facing(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 2),
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::CHAIR),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::CHAIR),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('▌')
            .with_foreground(colour::CHAIR),
    );
}

fn teapot(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 1),
        0,
        RenderCell::default()
            .with_character('┐')
            .with_bold(true)
            .with_foreground(colour::TEAPOT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(2, 2),
        0,
        RenderCell::default()
            .with_character('┘')
            .with_bold(true)
            .with_foreground(colour::TEAPOT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 1),
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::TEAPOT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(1, 2),
        0,
        RenderCell::default()
            .with_character('█')
            .with_foreground(colour::TEAPOT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord::new(0, 1),
        0,
        RenderCell::default()
            .with_character('╲')
            .with_bold(true)
            .with_foreground(colour::TEAPOT),
    );
}

fn tea(pattern: u16, ctx: Ctx, fb: &mut FrameBuffer) {
    for (i, coord) in Size::new_u16(3, 3).coord_iter_row_major().enumerate() {
        if pattern & (1 << i) != 0 {
            fb.set_cell_relative_to_ctx(
                ctx,
                coord,
                0,
                RenderCell::default()
                    .with_character('♣')
                    .with_bold(true)
                    .with_foreground(colour::TEA),
            );
        }
    }
}
