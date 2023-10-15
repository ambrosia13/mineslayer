use array2d::Array2D;
use bevy::prelude::*;
use rand::prelude::*;

pub const MAP_SIZE: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Neighbor(u32),
    Mine,
}

#[derive(Component)]
pub struct Map {
    pub mine_count: u32,
    pub tiles: Array2D<Tile>,
}

impl Map {
    pub fn new(mine_count: u32) -> Self {
        Self {
            mine_count,
            tiles: Self::generate(mine_count),
        }
    }

    pub fn get_at(&self, index: (usize, usize)) -> Tile {
        self.tiles[(index.0, index.1)]
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

    fn index(tiles: &Array2D<Tile>, x: isize, y: isize) -> Option<&Tile> {
        tiles.get(x as usize, y as usize)
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