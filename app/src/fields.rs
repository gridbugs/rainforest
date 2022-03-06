use grid_2d::{Coord, Grid, Size};
use rand::{
    distributions::{uniform::Uniform, Distribution},
    Rng,
};
use rgb_int::Rgb24;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GroundField {
    grid: Grid<Rgb24>,
}

impl GroundField {
    fn choose<R: Rng>(rng: &mut R) -> Rgb24 {
        let r = Uniform::<u8>::new_inclusive(40, 90).sample(rng);
        let g = Uniform::<u8>::new_inclusive(80, 120).sample(rng);
        let b = Uniform::<u8>::new_inclusive(0, 30).sample(rng);
        Rgb24 { r, g, b }
    }
    pub fn new<R: Rng>(size: Size, rng: &mut R) -> Self {
        Self {
            grid: Grid::new_fn(size, |_| Self::choose(rng)),
        }
    }
    pub fn get(&self, coord: Coord) -> Option<Rgb24> {
        self.grid.get(coord).cloned()
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
