use crate::colour;
use chargrid::core::prelude::*;
use grid_2d::coord_2d::{Axis, Coord, Size};
use rainforest_game::{EntityTile, Game, Tile, VisibilityCell};

pub fn render_3x3_from_visibility(
    coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    let ctx = ctx.add_offset(coord * 3);
    let mut render_tile = |entity, tile, ctx| match tile {
        Tile::Wall => {
            let below = coord + Coord::new(0, 1);
            if let Some(render_cell) = game.visibility_grid().get_cell(below) {
                if render_cell.tile_layers().feature.is_some() {
                    wall_top(ctx, fb);
                } else {
                    wall_front(ctx, fb);
                }
            } else {
                wall_front(ctx, fb);
            }
        }
        Tile::Floor => floor(ctx, fb),
        Tile::Ground => ground(ctx, fb),
        Tile::Player => player(ctx, fb),
        Tile::Tree => tree(ctx, fb),
        Tile::Window(Axis::Y) => {
            let below = coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), ctx, fb);
        }
        Tile::Window(Axis::X) => window_x(ctx, fb),
        Tile::DoorOpen(Axis::X) => door_open_x(ctx, fb),
        Tile::DoorOpen(Axis::Y) => door_open_y(ctx, fb),
        Tile::DoorClosed(Axis::X) => door_closed_x(ctx, fb),
        Tile::DoorClosed(Axis::Y) => door_closed_y(ctx, fb),
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
    coord: Coord,
    visibility_cell: &VisibilityCell,
    game: &Game,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    let ctx = ctx.add_offset(coord * 3);
    let mut render_tile = |tile, ctx| match tile {
        Tile::Wall => {
            let below = coord + Coord::new(0, 1);
            if let Some(render_cell) = game.visibility_grid().get_cell(below) {
                if render_cell.tile_layers().feature.is_some() {
                    wall_top(ctx, fb);
                } else {
                    wall_front(ctx, fb);
                }
            } else {
                wall_front(ctx, fb);
            }
        }
        Tile::Floor => floor(ctx, fb),
        Tile::Ground => ground(ctx, fb),
        Tile::Tree => tree(ctx, fb),
        Tile::Player => player(ctx, fb),
        Tile::Window(Axis::Y) => {
            let below = coord + Coord::new(0, 1);
            window_y(game.contains_floor(below), ctx, fb);
        }
        Tile::Window(Axis::X) => window_x(ctx, fb),
        Tile::DoorOpen(Axis::X) => door_open_x(ctx, fb),
        Tile::DoorOpen(Axis::Y) => door_open_y(ctx, fb),
        Tile::DoorClosed(Axis::X) => door_closed_x(ctx, fb),
        Tile::DoorClosed(Axis::Y) => door_closed_y(ctx, fb),
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
            .with_character(' ')
            .with_background(colour::FLOOR_FOREGROUND),
    );
}

fn ground(ctx: Ctx, fb: &mut FrameBuffer) {
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
            .with_character(' ')
            .with_background(colour::GROUND_FOREGROUND),
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

fn window_y(floor_below: bool, ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::WALL_TOP),
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::WALL_FRONT),
        );
    }
    if floor_below {
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            fb.set_cell_relative_to_ctx(
                ctx,
                offset + Coord { x: 0, y: 0 },
                0,
                RenderCell::default()
                    .with_character('▄')
                    .with_foreground(colour::WALL_FRONT),
            );
        }
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            fb.set_cell_relative_to_ctx(
                ctx,
                offset + Coord { x: 0, y: 2 },
                0,
                RenderCell::default()
                    .with_character('▄')
                    .with_foreground(colour::FLOOR_BACKGROUND),
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
                .with_character('▌')
                .with_background(colour::WINDOWS)
                .with_foreground(colour::WALL_FRONT),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 2, y: 1 },
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(colour::WALL_FRONT)
                .with_foreground(colour::WINDOWS),
        );
    } else {
        for offset in Size::new_u16(3, 1).coord_iter_row_major() {
            fb.set_cell_relative_to_ctx(
                ctx,
                offset + Coord { x: 0, y: 0 },
                0,
                RenderCell::default()
                    .with_character('▀')
                    .with_foreground(colour::FLOOR_BACKGROUND),
            );
        }
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 1, y: 1 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colour::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 1, y: 2 },
            0,
            RenderCell::default()
                .with_character('▀')
                .with_foreground(colour::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character('▗')
                .with_foreground(colour::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 2, y: 1 },
            0,
            RenderCell::default()
                .with_character('▖')
                .with_foreground(colour::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 0, y: 2 },
            0,
            RenderCell::default()
                .with_character('▝')
                .with_foreground(colour::WINDOWS),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            Coord { x: 2, y: 2 },
            0,
            RenderCell::default()
                .with_character('▘')
                .with_foreground(colour::WINDOWS),
        );
    }
}

fn window_x(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::WALL_TOP),
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
            .with_character('▌')
            .with_background(colour::WINDOWS)
            .with_foreground(colour::WALL_TOP),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
            .with_background(colour::WALL_TOP)
            .with_foreground(colour::WINDOWS),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colour::WALL_FRONT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colour::WALL_FRONT),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colour::WALL_FRONT),
    );
}

fn door_closed_y(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::DOOR),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 0 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colour::DOOR_BORDER)
                .with_background(colour::FLOOR_BACKGROUND),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 2 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colour::FLOOR_BACKGROUND)
                .with_background(colour::DOOR_BORDER),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 1 },
        0,
        RenderCell::default()
            .with_character('▌')
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
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('│')
            .with_foreground(colour::DOOR_BORDER)
            .with_bold(true),
    );
}

fn door_closed_x(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(1, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 1, y: 0 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::DOOR),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 0 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::FLOOR_BACKGROUND),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 0 },
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(colour::DOOR_BORDER)
                .with_foreground(colour::FLOOR_BACKGROUND),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 2, y: 0 },
            0,
            RenderCell::default()
                .with_character('▌')
                .with_background(colour::FLOOR_BACKGROUND)
                .with_foreground(colour::DOOR_BORDER),
        );
    }
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 1 },
        0,
        RenderCell::default()
            .with_character('─')
            .with_foreground(colour::DOOR_BORDER)
            .with_bold(true),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::DOOR_BORDER),
    );
}

fn door_open_y(ctx: Ctx, fb: &mut FrameBuffer) {
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
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colour::DOOR_BORDER),
    );
}

fn door_open_x(ctx: Ctx, fb: &mut FrameBuffer) {
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 0 },
        0,
        RenderCell::default()
            .with_character('▘')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 0 },
        0,
        RenderCell::default()
            .with_character('▝')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 2, y: 2 },
        0,
        RenderCell::default()
            .with_character('▖')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 0, y: 2 },
        0,
        RenderCell::default()
            .with_character('▗')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 0 },
        0,
        RenderCell::default()
            .with_character('▀')
            .with_foreground(colour::DOOR_BORDER),
    );
    fb.set_cell_relative_to_ctx(
        ctx,
        Coord { x: 1, y: 2 },
        0,
        RenderCell::default()
            .with_character('▄')
            .with_foreground(colour::DOOR_BORDER),
    );
}

pub fn wall_top(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::WALL_TOP),
        );
    }
}

pub fn wall_front(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::WALL_TOP),
        );
    }
    for offset in Size::new_u16(3, 2).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character(' ')
                .with_background(colour::WALL_FRONT),
        );
    }
    for offset in Size::new_u16(3, 1).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 1 },
            0,
            RenderCell::default()
                .with_character('▄')
                .with_foreground(colour::STRIPE),
        );
        fb.set_cell_relative_to_ctx(
            ctx,
            offset + Coord { x: 0, y: 2 },
            0,
            RenderCell::default()
                .with_character('▀')
                .with_foreground(colour::STRIPE),
        );
    }
}

pub fn tree(ctx: Ctx, fb: &mut FrameBuffer) {
    for offset in Size::new_u16(3, 3).coord_iter_row_major() {
        fb.set_cell_relative_to_ctx(
            ctx,
            offset,
            0,
            RenderCell::default()
                .with_character('?')
                .with_background(colour::WALL_TOP),
        );
    }
}
