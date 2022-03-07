use crate::{
    fields::{GroundField, LogField, TeaField},
    mist::Mist,
    tile_3x3,
};
use chargrid::{
    core::rgb_int::{rgb24, Rgb24},
    prelude::*,
};
use rainforest_game::{CellVisibility, Game};

#[derive(Clone, Copy)]
struct Remembered {
    mist_colour: Rgba32,
}
impl Tint for Remembered {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        let mean = rgba32
            .to_rgb24()
            .weighted_mean_u16(rgb24::WeightsU16::new(1, 1, 1));
        self.mist_colour
            .saturating_scalar_mul_div(2, 3)
            .alpha_composite(Rgba32::new_grey(mean).saturating_scalar_mul_div(1, 2))
    }
}

#[derive(Clone, Copy)]
struct LightBlend {
    light_colour: Rgb24,
    mist_colour: Rgba32,
}

impl Tint for LightBlend {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        self.mist_colour
            .alpha_composite(rgba32)
            .to_rgb24()
            .normalised_mul(self.light_colour)
            .saturating_add(self.light_colour.saturating_scalar_mul_div(1, 10))
            .to_rgba32(255)
    }
}

pub fn render_game_with_visibility(
    game: &Game,
    offset: Coord,
    size: Size,
    ground_field: &GroundField,
    log_field: &LogField,
    tea_field: &TeaField,
    mist: &Mist,
    ctx: Ctx,
    fb: &mut FrameBuffer,
) {
    let visibility_grid = game.visibility_grid();
    let vis_count = visibility_grid.count();
    for screen_coord in size.coord_iter_row_major() {
        let world_coord = offset + screen_coord;
        if let Some(visibility_cell) = visibility_grid.get_cell(world_coord) {
            let mist_colour = if game.should_hide_rain(world_coord) {
                Rgba32::new(0, 0, 0, 0)
            } else {
                mist.get(world_coord)
            };
            match visibility_cell.visibility(vis_count) {
                CellVisibility::CurrentlyVisibleWithLightColour(Some(light_colour)) => {
                    tile_3x3::render_3x3_from_visibility(
                        screen_coord,
                        world_coord,
                        visibility_cell,
                        game,
                        ground_field,
                        log_field,
                        tea_field,
                        ctx_tint!(
                            ctx,
                            LightBlend {
                                light_colour,
                                mist_colour
                            }
                        ),
                        fb,
                    );
                }
                CellVisibility::PreviouslyVisible => {
                    tile_3x3::render_3x3_from_visibility_remembered(
                        screen_coord,
                        world_coord,
                        visibility_cell,
                        game,
                        log_field,
                        tea_field,
                        ctx_tint!(ctx, Remembered { mist_colour }),
                        fb,
                    );
                }
                CellVisibility::NeverVisible
                | CellVisibility::CurrentlyVisibleWithLightColour(None) => (),
            }
        }
    }
}
