use crate::maze_gen::{MazeGenerator, MAP_WIDTH, MAP_HEIGHT};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WallType {
    Empty = 0,
    Red = 1,
    Green = 2,
    Blue = 3,
    White = 4,
    Yellow = 5,
}

impl WallType {
    #[allow(dead_code)]
    pub fn color(&self) -> u8 {
        match self {
            WallType::Empty => 0,
            WallType::Red => 1,
            WallType::Green => 2,
            WallType::Blue => 3,
            WallType::White => 4,
            WallType::Yellow => 5,
        }
    }
}

pub struct World {
    map: [[WallType; MAP_HEIGHT]; MAP_WIDTH],
    pub width: usize,
    pub height: usize,
    start_pos: (f64, f64),
}

impl World {
    pub fn new_random() -> Self {
        let mut generator = MazeGenerator::new();
        let map = generator.generate();
        let start_pos = generator.get_start_position();
        
        World { 
            map,
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            start_pos,
        }
    }

    pub fn get_start_position(&self) -> (f64, f64) {
        self.start_pos
    }

    pub fn get(&self, x: i32, y: i32) -> WallType {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return WallType::Red;
        }
        self.map[x as usize][y as usize]
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        self.get(x, y) != WallType::Empty
    }
    
    pub fn get_map(&self) -> &[[WallType; MAP_HEIGHT]; MAP_WIDTH] {
        &self.map
    }
}
