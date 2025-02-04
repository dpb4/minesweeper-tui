use std::fmt;

pub mod lose_menu;
pub mod option_menu;
pub mod win_menu;

#[derive(Clone, Debug)]
struct Tile {
    hidden: bool,
    flag: bool,
    mine: bool,
    count: u8,
}

pub enum TileState {
    Hidden,
    Flagged,
    Empty,
    Mine,
    Count(u8),
}

#[derive(Debug)]
pub struct Board {
    grid: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
    pub mine_count: u32,
    pub mines_left: i32,
}

impl Tile {
    fn new(mine: bool) -> Self {
        Self {
            hidden: true,
            flag: false,
            mine,
            count: 0,
        }
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.hidden {
            write!(f, "{}", if self.flag { "◄" } else { "◼" })
        } else if self.mine {
            write!(f, "◉")
        } else if self.count == 0 {
            write!(f, " ")
        } else {
            write!(f, "{}", self.count)
        }
    }
}

impl TileState {
    fn new(tile: &Tile) -> TileState {
        if tile.hidden {
            if tile.flag {
                TileState::Flagged
            } else {
                TileState::Hidden
            }
        } else if tile.mine {
            TileState::Mine
        } else {
            if tile.count == 0 {
                TileState::Empty
            } else {
                TileState::Count(tile.count)
            }
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(20, 20, 50)
    }
}

const SAFETY_RADIUS: i32 = 2;

impl Board {
    pub fn new(width: usize, height: usize, mine_count: u32) -> Self {
        Self {
            grid: vec![vec![Tile::new(false); width]; height],
            width,
            height,
            mine_count,
            mines_left: mine_count as i32,
        }
    }

    pub fn first_dig(&mut self, x: usize, y: usize) {
        self.grid = self.generate_grid_safe(self.width, self.height, self.mine_count, x, y);
        let _ = self.dig(x, y);
    }

    pub fn dig(&mut self, x: usize, y: usize) -> Result<(), ()> {
        if self.grid[y][x].flag {
            return Ok(());
        }

        if !self.grid[y][x].mine && self.grid[y][x].count == 0 {
            self.flood_dig(x, y);
        } else if self.grid[y][x].hidden {
            self.grid[y][x].hidden = false;
            if self.grid[y][x].mine {
                return Err(());
            }
        } else if self.is_valid_smart_clear(x, y) {
            self.smart_clear(x, y)?;
        }

        Ok(())
    }

    fn in_bounds(&self, x: i8, y: i8) -> bool {
        x >= 0 && x < (self.width as i8) && y >= 0 && y < (self.height as i8)
    }

    pub fn undo(&mut self, x: usize, y: usize) {
        let [ix, iy] = [x as i8, y as i8];
        let dirs = [-1, 0, 1];

        for dx in dirs {
            for dy in dirs {
                if self.in_bounds(ix + dx, iy + dy) {
                    let tile = &mut self.grid[(iy + dy) as usize][(ix + dx) as usize];

                    if !tile.hidden && tile.mine {
                        tile.hidden = true;
                    }
                }
            }
        }
    }

    pub fn game_won(&self) -> bool {
        if self.mines_left != 0 {
            return false;
        }

        self.grid
            .iter()
            .flatten()
            .all(|t| (t.mine && t.flag) || !t.hidden)
    }

    pub fn flag(&mut self, x: usize, y: usize) {
        if self.grid[y][x].hidden {
            if self.grid[y][x].flag {
                self.grid[y][x].flag = false;
                self.mines_left += 1;
            } else {
                self.grid[y][x].flag = true;
                self.mines_left -= 1;
            }
        }
    }

    pub fn check(&self, x: usize, y: usize) -> TileState {
        TileState::new(&self.grid[y][x])
    }

    fn is_valid_smart_clear(&self, x: usize, y: usize) -> bool {
        if self.grid[y][x].count == 0 || self.grid[y][x].hidden {
            return false;
        }

        let mut count = 0;
        let [ix, iy] = [x as i8, y as i8];
        let dirs = [-1, 0, 1];

        for dx in dirs {
            for dy in dirs {
                if !(dx == 0 && dy == 0) && self.in_bounds(ix + dx, iy + dy) {
                    let tile = &self.grid[(iy + dy) as usize][(ix + dx) as usize];

                    if tile.hidden && tile.flag {
                        count += 1;
                    }
                }
            }
        }

        self.grid[y][x].count == count
    }

    fn smart_clear(&mut self, x: usize, y: usize) -> Result<(), ()> {
        let [ix, iy] = [x as i8, y as i8];
        let dirs = [-1, 0, 1];

        for dx in dirs {
            for dy in dirs {
                if !(dx == 0 && dy == 0) && self.in_bounds(ix + dx, iy + dy) {
                    let tile = &self.grid[(iy + dy) as usize][(ix + dx) as usize];

                    if tile.hidden && !tile.flag {
                        self.dig((ix + dx) as usize, (iy + dy) as usize)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn flood_dig(&mut self, x: usize, y: usize) {
        let tile = &mut self.grid[y][x];

        if tile.count != 0 || !tile.hidden && !tile.mine {
            tile.hidden = false;
            return;
        }
        tile.hidden = false;

        let [ix, iy] = [x as i8, y as i8];
        let dirs = [-1, 0, 1];

        for dx in dirs {
            for dy in dirs {
                if !(dx == 0 && dy == 0) && self.in_bounds(ix + dx, iy + dy) {
                    self.flood_dig((ix + dx) as usize, (iy + dy) as usize);
                }
            }
        }
    }

    fn generate_grid_safe(
        &self,
        width: usize,
        height: usize,
        mine_count: u32,
        dig_x: usize,
        dig_y: usize,
    ) -> Vec<Vec<Tile>> {
        let mut grid = vec![vec![Tile::new(false); width]; height];

        for _ in 0..mine_count {
            let mut x = ((rand::random::<f32>() * width as f32) as usize).clamp(0, width - 1);
            let mut y = ((rand::random::<f32>() * height as f32) as usize).clamp(0, height - 1);

            while grid[y][x].mine || Self::mine_too_close(x, y, dig_x, dig_y) {
                x = (rand::random::<f32>() * width as f32) as usize;
                y = (rand::random::<f32>() * height as f32) as usize;
            }

            grid[y][x] = Tile::new(true);
        }

        self.count_mines(&mut grid, width, height);
        return grid;
    }

    fn mine_too_close(x: usize, y: usize, dig_x: usize, dig_y: usize) -> bool {
        let dx = x as i32 - dig_x as i32;
        let dy = y as i32 - dig_y as i32;
        dx * dx + dy * dy <= SAFETY_RADIUS * SAFETY_RADIUS
    }

    fn count_mines(&self, grid: &mut Vec<Vec<Tile>>, width: usize, height: usize) {
        for y in 0..height {
            for x in 0..width {
                if grid[y][x].mine {
                    continue;
                }

                let [ix, iy] = [x as i8, y as i8];

                let dirs = [-1, 0, 1];
                let mut mine_count = 0;

                for dx in dirs {
                    for dy in dirs {
                        if !(dx == 0 && dy == 0) && self.in_bounds(ix + dx, iy + dy) {
                            if grid[(iy + dy) as usize][(ix + dx) as usize].mine {
                                mine_count += 1;
                            }
                        }
                    }
                }

                grid[y][x].count = mine_count;
            }
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let _ = write!(
                    f,
                    "{}{}",
                    self.grid[y][x],
                    if x < self.width - 1 { " " } else { "" }
                );
            }
            let _ = write!(f, "\n");
        }

        Ok(())
    }
}
