use crate::colour;
use chargrid::prelude::*;
use grid_2d::{Coord, Grid, Size};
use rainforest_game::Game;
use rand::{Rng, SeedableRng};
use rand_isaac::Isaac64Rng;
use serde::{Deserialize, Serialize};

const MAX_NUM_DROPS: usize = 10000;
const SPASH_DURATION: u32 = 0;
const DROP_INTERVAL: u32 = 1;

#[derive(Serialize, Deserialize)]
pub enum RainDirection {
    Diagonal,
    Vertical,
}

#[derive(Serialize, Deserialize)]
enum RainDropState {
    Falling,
    Splash,
}

#[derive(Serialize, Deserialize)]
struct RainDrop {
    coord: Coord,
    remaining: u32,
    state: RainDropState,
}

#[derive(Serialize, Deserialize)]
pub struct Rain {
    drops: Vec<RainDrop>,
    num_drops: usize,
    direction: RainDirection,
    size: Size,
    rng: Isaac64Rng,
    remaining: u32,
    hide_table: Grid<bool>,
}

impl Rain {
    pub fn new<R: Rng>(
        game: &Game,
        num_drops: usize,
        direction: RainDirection,
        rng: &mut R,
    ) -> Self {
        let size = game.world_size() * 3;
        let mut drops = Vec::with_capacity(MAX_NUM_DROPS);
        for _ in 0..MAX_NUM_DROPS {
            drops.push(RainDrop {
                coord: Coord::new(
                    rng.gen_range(0..(size.width() as i32 * 2)),
                    rng.gen_range(0..(size.height() as i32)),
                ),
                remaining: rng.gen_range(0..size.height()),
                state: RainDropState::Falling,
            });
        }
        let rng = Isaac64Rng::from_rng(rng).unwrap();
        let remaining = DROP_INTERVAL;
        let hide_table = Grid::new_fn(game.world_size(), |coord| game.should_hide_rain(coord));
        Self {
            drops,
            num_drops,
            direction,
            size,
            rng,
            remaining,
            hide_table,
        }
    }

    pub fn tick(&mut self) {
        if let Some(remaining) = self.remaining.checked_sub(1) {
            self.remaining = remaining;
            return;
        }
        let step = match self.direction {
            RainDirection::Vertical => Coord::new(0, 1),
            RainDirection::Diagonal => Coord::new(-1, 1),
        };
        self.remaining = DROP_INTERVAL;
        for drop in &mut self.drops[0..self.num_drops] {
            match drop.state {
                RainDropState::Falling => {
                    drop.coord += step;
                    if let Some(remaining) = drop.remaining.checked_sub(1) {
                        drop.remaining = remaining;
                    } else {
                        drop.state = RainDropState::Splash;
                        drop.remaining = SPASH_DURATION;
                    }
                }
                RainDropState::Splash => {
                    if let Some(remaining) = drop.remaining.checked_sub(1) {
                        drop.remaining = remaining;
                    } else {
                        drop.state = RainDropState::Falling;
                        drop.remaining = self.rng.gen_range(0..self.size.height());
                        drop.coord =
                            Coord::new(self.rng.gen_range(0..(self.size.width() as i32 * 2)), 0);
                    }
                }
            }
        }
    }

    pub fn render(&self, game: &Game, offset: Coord, size: Size, ctx: Ctx, fb: &mut FrameBuffer) {
        let offset = offset * 3;
        let size = size * 3;
        let falling_cell = match self.direction {
            RainDirection::Vertical => RenderCell::default().with_character('.').with_bold(true),
            RainDirection::Diagonal => RenderCell::default().with_character(',').with_bold(true),
        };
        let visibility_grid = game.visibility_grid();
        for drop in &self.drops[0..self.num_drops] {
            let world_coord = drop.coord / 3;
            if self.hide_table.get(world_coord) == Some(&true) {
                continue;
            }
            if visibility_grid.is_coord_never_visible(world_coord) {
                continue;
            }
            let colour = if visibility_grid.is_coord_currently_visible(world_coord) {
                colour::RAIN
            } else {
                colour::RAIN_REMEMBERED
            };
            let coord = drop.coord - offset;
            if coord.is_valid(size) {
                match drop.state {
                    RainDropState::Falling => fb.set_cell_relative_to_ctx(
                        ctx,
                        coord,
                        10,
                        falling_cell.with_foreground(colour),
                    ),
                    RainDropState::Splash => {
                        fb.set_cell_relative_to_ctx(
                            ctx,
                            coord,
                            10,
                            RenderCell::default()
                                .with_character('…')
                                .with_bold(true)
                                .with_foreground(colour),
                        );
                    }
                }
            }
        }
    }
}
