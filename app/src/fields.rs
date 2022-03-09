use chargrid::prelude::*;
use grid_2d::{Coord, Grid, Size};
use rand::{
    distributions::{uniform::Uniform, Distribution},
    seq::SliceRandom,
    Rng,
};
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TeaField {
    grid: Grid<u16>,
}

impl TeaField {
    pub fn new<R: Rng>(size: Size, rng: &mut R) -> Self {
        let candidates = (0..=512u16)
            .filter(|u| u.count_ones() > 4 && u.count_ones() < 7)
            .collect::<Vec<_>>();
        let grid = Grid::new_fn(size, |_| *candidates.choose(rng).unwrap());
        Self { grid }
    }

    pub fn get(&self, coord: Coord) -> Option<u16> {
        self.grid.get(coord).cloned()
    }
}

#[derive(Serialize, Deserialize)]
struct GroundCell {
    fg: Rgb24,
    bg: Rgb24,
    ch: char,
}

#[derive(Serialize, Deserialize)]
pub struct GroundField {
    grid: Grid<GroundCell>,
}

impl GroundField {
    fn choose<R: Rng>(coord: Coord, rng: &mut R) -> GroundCell {
        let r = Uniform::<u8>::new_inclusive(40, 90).sample(rng);
        let g = Uniform::<u8>::new_inclusive(80, 120).sample(rng);
        let b = Uniform::<u8>::new_inclusive(0, 30).sample(rng);
        let colour = Rgb24 { r, g, b };
        let (ch, fg, bg) = if coord.x % 3 == 1 && coord.y % 3 == 1 {
            ('▪', colour, colour.saturating_scalar_mul_div(1, 5))
        } else {
            (
                *[' ', ' ', ' ', ' ', '`', ',', '\'', ';']
                    .choose(rng)
                    .unwrap(),
                colour.saturating_scalar_mul_div(1, 2),
                colour.saturating_scalar_mul_div(1, 5),
            )
        };
        GroundCell { fg, bg, ch }
    }
    pub fn new<R: Rng>(size: Size, rng: &mut R) -> Self {
        Self {
            grid: Grid::new_fn(size * 3, |coord| Self::choose(coord, rng)),
        }
    }
    pub fn render(&self, coord: Coord, ctx: Ctx, fb: &mut FrameBuffer) {
        let base_coord = coord * 3;
        for offset in Size::new_u16(3, 3).coord_iter_row_major() {
            let cell = self.grid.get(base_coord + offset).unwrap();
            fb.set_cell_relative_to_ctx(
                ctx,
                offset,
                0,
                RenderCell::default()
                    .with_character(cell.ch)
                    .with_foreground(cell.fg.to_rgba32(255))
                    .with_background(cell.bg.to_rgba32(255)),
            );
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LogField {
    horizontal: Grid<Rgb24>,
    vertical: Grid<Rgb24>,
}

impl LogField {
    fn choose<R: Rng>(rng: &mut R) -> Rgb24 {
        let r = Uniform::<u8>::new_inclusive(60, 80).sample(rng);
        let g = Uniform::<u8>::new_inclusive(40, 50).sample(rng);
        let b = Uniform::<u8>::new_inclusive(0, 20).sample(rng);
        Rgb24 { r, g, b }
    }
    fn new_horizontal<R: Rng>(size: Size, rng: &mut R) -> Grid<Rgb24> {
        let mut current_colour = Rgb24::new_grey(0);
        let mut count = 0;
        Grid::new_fn(size, |coord| {
            if coord.x == 0 || count == 0 {
                if coord.x == 0 && coord.y % 2 == 0 {
                    count = 5;
                } else {
                    count = 10;
                }
                current_colour = Self::choose(rng);
            } else {
                count -= 1;
            }
            current_colour
        })
    }
    pub fn new<R: Rng>(size: Size, rng: &mut R) -> Self {
        let horizontal = Self::new_horizontal(Size::new(size.width() * 3, size.height() * 6), rng);
        let vertical = Self::new_horizontal(
            Size::new(size.width() * 6, size.height() * 3).transpose(),
            rng,
        )
        .transpose_clone();
        Self {
            horizontal,
            vertical,
        }
    }
    pub fn get_horizontal(&self, coord: Coord) -> Option<Rgb24> {
        self.horizontal.get(coord).cloned()
    }
    pub fn get_vertical(&self, coord: Coord) -> Option<Rgb24> {
        self.vertical.get(coord).cloned()
    }
}
