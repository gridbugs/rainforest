use crate::tile_3x3;
use chargrid::{
    core::rgb_int::{rgb24, Rgb24},
    prelude::*,
};
use rainforest_game::{CellVisibility, Game, Layer};

#[derive(Clone, Copy)]
struct Remembered;
impl Tint for Remembered {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        let mean = rgba32
            .to_rgb24()
            .weighted_mean_u16(rgb24::WeightsU16::new(1, 1, 1));
        Rgb24::new_grey(mean)
            .saturating_scalar_mul_div(1, 2)
            .to_rgba32(255)
    }
}

#[derive(Clone, Copy)]
struct LightBlend {
    light_colour: Rgb24,
}

impl Tint for LightBlend {
    fn tint(&self, rgba32: Rgba32) -> Rgba32 {
        rgba32
            .to_rgb24()
            .normalised_mul(self.light_colour)
            .saturating_add(self.light_colour.saturating_scalar_mul_div(1, 10))
            .to_rgba32(255)
    }
}

pub fn render_game_with_visibility(game: &Game, ctx: Ctx, fb: &mut FrameBuffer) {
    let vis_count = game.visibility_grid().count();
    for (coord, visibility_cell) in game.visibility_grid().enumerate() {
        match visibility_cell.visibility(vis_count) {
            CellVisibility::CurrentlyVisibleWithLightColour(Some(light_colour)) => {
                tile_3x3::render_3x3_from_visibility(
                    coord,
                    visibility_cell,
                    game,
                    ctx_tint!(ctx, LightBlend { light_colour }),
                    fb,
                );
            }
            CellVisibility::PreviouslyVisible => {
                tile_3x3::render_3x3_from_visibility_remembered(
                    coord,
                    visibility_cell,
                    game,
                    ctx_tint!(ctx, Remembered),
                    fb,
                );
            }
            CellVisibility::NeverVisible
            | CellVisibility::CurrentlyVisibleWithLightColour(None) => (),
        }
    }
}
