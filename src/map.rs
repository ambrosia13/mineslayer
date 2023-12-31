use array2d::Array2D;
use bevy::prelude::*;
use rand::prelude::*;

pub const MAP_SIZE: usize = 40;
pub const MINE_COUNT: u32 = 120;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Neighbor(u32),
    Mine,
}

impl Tile {
    pub fn get_color(self) -> Color {
        match self {
            Tile::Empty => Color::WHITE,
            Tile::Neighbor(_) => Color::YELLOW,
            Tile::Mine => Color::RED,
        }
    }
}

#[derive(Component)]
pub struct Map {
    pub mine_count: u32,
    pub tiles: Array2D<Tile>,
    pub visibility: Array2D<bool>,
}

pub struct TileDisplay(pub bool, pub Tile);

impl TileDisplay {
    pub fn get_color(&self) -> Color {
        if !self.0 {
            Color::GRAY
        } else {
            self.1.get_color()
        }
    }
}

impl Map {
    pub fn new(mine_count: u32) -> Self {
        Self {
            mine_count,
            tiles: Self::generate(mine_count),
            visibility: Array2D::filled_with(false, MAP_SIZE, MAP_SIZE),
        }
    }

    pub fn get_at(&self, index: (usize, usize)) -> TileDisplay {
        TileDisplay(
            self.visibility[(index.0, index.1)],
            self.tiles[(index.0, index.1)],
        )
    }

    pub fn set_visibility_at(&mut self, index: (usize, usize), visibility: bool) {
        self.visibility[index] = visibility;
    }

    /// Does one step of propagating visible tiles. If there was any visibility change, returns true. Otherwise returns false.
    pub fn propagate_visibility(&mut self) -> bool {
        let directions = [(0, 1), (0, -1), (-1, 0), (1, 0)];

        let mut new_visibility = self.visibility.clone();

        for x in 0..(MAP_SIZE as isize) {
            for y in 0..(MAP_SIZE as isize) {
                let should_propagate = directions
                    .iter()
                    .filter_map(|(dx, dy)| {
                        let row = (x + dx) as usize;
                        let column = (y + dy) as usize;

                        let tile = self.tiles.get(row, column);

                        if let Some(tile) = tile {
                            let visible = self.visibility.get(row, column);

                            if *tile == Tile::Empty {
                                visible
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .any(|visible| *visible);

                if should_propagate {
                    new_visibility[(x as usize, y as usize)] = true;
                }
            }
        }

        if self.visibility != new_visibility {
            self.visibility = new_visibility;
            true
        } else {
            false
        }
    }

    fn place_bombs(tiles: &mut Array2D<Tile>, mine_count: u32) {
        let mut mines_left = mine_count;
        while mines_left > 0 {
            let x = thread_rng().gen_range(0..MAP_SIZE);
            let y = thread_rng().gen_range(0..MAP_SIZE);

            if tiles[(x, y)] == Tile::Empty {
                tiles[(x, y)] = Tile::Mine;
                mines_left -= 1;
            }
        }
    }

    fn count_neighbors(tiles: &mut Array2D<Tile>, x: isize, y: isize) {
        let directions = [
            (0, 1),
            (0, -1),
            (-1, 0),
            (1, 0),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        let neighbors = directions
            .iter()
            .map(|(dx, dy)| tiles.get((x + dx) as usize, (y + dy) as usize))
            .filter(|tile| tile.is_some_and(|tile| *tile == Tile::Mine))
            .count();

        let (x, y) = (x as usize, y as usize);

        if neighbors > 0 && tiles[(x, y)] == Tile::Empty {
            tiles[(x, y)] = Tile::Neighbor(neighbors as u32);
        }
    }

    fn generate(mine_count: u32) -> Array2D<Tile> {
        let mut tiles = Array2D::filled_with(Tile::Empty, MAP_SIZE, MAP_SIZE);

        // Place bombs
        Self::place_bombs(&mut tiles, mine_count);

        // Count neighbors
        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                Self::count_neighbors(&mut tiles, x as isize, y as isize);
            }
        }

        tiles
    }
}
