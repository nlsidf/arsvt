use rand::Rng;
use crate::world::WallType;

pub const MAP_WIDTH: usize = 51;
pub const MAP_HEIGHT: usize = 51;

pub struct MazeGenerator {
    map: [[bool; MAP_HEIGHT]; MAP_WIDTH],
}

impl MazeGenerator {
    pub fn new() -> Self {
        MazeGenerator {
            map: [[true; MAP_HEIGHT]; MAP_WIDTH],
        }
    }

    pub fn generate(&mut self) -> [[WallType; MAP_HEIGHT]; MAP_WIDTH] {
        let mut rng = rand::thread_rng();
        
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                self.map[x][y] = true;
            }
        }

        self.carve_path(1, 1, &mut rng);

        let mut result = [[WallType::Empty; MAP_HEIGHT]; MAP_WIDTH];
        
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if self.map[x][y] {
                    let wall_type = if x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1 {
                        WallType::Red
                    } else {
                        let pattern = (x / 5 + y / 5) % 5;
                        match pattern {
                            0 => WallType::Red,
                            1 => WallType::Green,
                            2 => WallType::Blue,
                            3 => WallType::White,
                            _ => WallType::Yellow,
                        }
                    };
                    result[x][y] = wall_type;
                }
            }
        }

        result
    }

    fn carve_path(&mut self, x: usize, y: usize, rng: &mut impl Rng) {
        self.map[x][y] = false;

        let mut directions = [(0, -2), (0, 2), (-2, 0), (2, 0)];
        
        for i in (1..directions.len()).rev() {
            let j = rng.gen_range(0..=i);
            directions.swap(i, j);
        }

        for (dx, dy) in directions.iter() {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx > 0 && ny > 0 && nx < (MAP_WIDTH - 1) as i32 && ny < (MAP_HEIGHT - 1) as i32 {
                let nx = nx as usize;
                let ny = ny as usize;

                if self.map[nx][ny] {
                    let mx = (x as i32 + dx / 2) as usize;
                    let my = (y as i32 + dy / 2) as usize;
                    self.map[mx][my] = false;
                    
                    self.carve_path(nx, ny, rng);
                }
            }
        }
    }

    pub fn get_start_position(&self) -> (f64, f64) {
        let mut rng = rand::thread_rng();
        
        loop {
            let x = rng.gen_range(1..MAP_WIDTH - 1);
            let y = rng.gen_range(1..MAP_HEIGHT - 1);
            
            if !self.map[x][y] {
                return (x as f64 + 0.5, y as f64 + 0.5);
            }
        }
    }
}
