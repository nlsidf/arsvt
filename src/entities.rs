use crate::world::WallType;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum ItemType {
    Coin,
    Key,
    Health,
    Exit,
}

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub x: f64,
    pub y: f64,
    pub item_type: ItemType,
    pub collected: bool,
}

impl Item {
    pub fn new(x: f64, y: f64, item_type: ItemType) -> Self {
        Item {
            x,
            y,
            item_type,
            collected: false,
        }
    }

    #[allow(dead_code)]
    pub fn get_icon(&self) -> char {
        match self.item_type {
            ItemType::Coin => '◆',
            ItemType::Key => '🔑',
            ItemType::Health => '❤',
            ItemType::Exit => '🚪',
        }
    }

    #[allow(dead_code)]
    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NPC {
    pub x: f64,
    pub y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub npc_type: NPCType,
    pub animation_phase: f64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NPCType {
    Wanderer,
    Guard,
}

impl NPC {
    pub fn new(x: f64, y: f64, npc_type: NPCType) -> Self {
        let mut rng = rand::thread_rng();
        let angle = rng.gen_range(0.0..std::f64::consts::PI * 2.0);
        
        NPC {
            x,
            y,
            dir_x: angle.cos(),
            dir_y: angle.sin(),
            npc_type,
            animation_phase: 0.0,
        }
    }

    pub fn update(&mut self, world_map: &[[WallType; crate::maze_gen::MAP_HEIGHT]; crate::maze_gen::MAP_WIDTH], delta_time: f64) {
        self.animation_phase += delta_time * 3.0;
        
        let speed = match self.npc_type {
            NPCType::Wanderer => 0.02,
            NPCType::Guard => 0.01,
        };

        let new_x = self.x + self.dir_x * speed;
        let new_y = self.y + self.dir_y * speed;

        if world_map[new_x as usize][self.y as usize] == WallType::Empty {
            self.x = new_x;
        } else {
            self.dir_x = -self.dir_x;
        }

        if world_map[self.x as usize][new_y as usize] == WallType::Empty {
            self.y = new_y;
        } else {
            self.dir_y = -self.dir_y;
        }

        if rand::thread_rng().gen_range(0..100) < 2 {
            let angle = rand::thread_rng().gen_range(0.0..std::f64::consts::PI * 2.0);
            self.dir_x = angle.cos();
            self.dir_y = angle.sin();
        }
    }

    #[allow(dead_code)]
    pub fn get_sprite(&self) -> char {
        let phase = (self.animation_phase % 2.0) / 2.0;
        match self.npc_type {
            NPCType::Wanderer => {
                if phase < 0.25 { '🚶' }
                else if phase < 0.5 { '🚶' }
                else if phase < 0.75 { '🚶' }
                else { '🚶' }
            }
            NPCType::Guard => {
                if phase < 0.5 { '💂' } else { '💂' }
            }
        }
    }

    #[allow(dead_code)]
    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }
}
