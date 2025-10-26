use crate::vec2::Vec2;
use crate::world::World;
use std::f64::consts::PI;

pub struct Camera {
    pub position: Vec2,
    pub direction: Vec2,
    pub plane: Vec2,
    pub move_speed: f64,
    pub rot_speed: f64,
    pub pitch: f64,
    pub z_position: f64,
    pub z_velocity: f64,
    pub bob_phase: f64,
}

impl Camera {
    pub fn new(position: Vec2, direction: Vec2) -> Self {
        let direction = direction.normalize();
        let plane = Vec2::new(0.0, 0.66);
        
        Camera {
            position,
            direction,
            plane,
            move_speed: 0.15,
            rot_speed: 0.08,
            pitch: 0.0,
            z_position: 0.0,
            z_velocity: 0.0,
            bob_phase: 0.0,
        }
    }

    pub fn move_forward(&mut self, world: &World, delta: f64) {
        let new_pos = self.position + self.direction * (self.move_speed * delta);
        if !world.is_wall(new_pos.x as i32, self.position.y as i32) {
            self.position.x = new_pos.x;
        }
        if !world.is_wall(self.position.x as i32, new_pos.y as i32) {
            self.position.y = new_pos.y;
        }
        
        self.bob_phase += 0.2;
        
        if self.pitch > 0.1 {
            self.z_velocity += 0.05;
        }
    }

    pub fn move_backward(&mut self, world: &World, delta: f64) {
        let new_pos = self.position - self.direction * (self.move_speed * delta);
        if !world.is_wall(new_pos.x as i32, self.position.y as i32) {
            self.position.x = new_pos.x;
        }
        if !world.is_wall(self.position.x as i32, new_pos.y as i32) {
            self.position.y = new_pos.y;
        }
        
        self.bob_phase += 0.2;
    }

    pub fn strafe_left(&mut self, world: &World, delta: f64) {
        let right = Vec2::new(self.direction.y, -self.direction.x);
        let new_pos = self.position - right * (self.move_speed * delta);
        if !world.is_wall(new_pos.x as i32, self.position.y as i32) {
            self.position.x = new_pos.x;
        }
        if !world.is_wall(self.position.x as i32, new_pos.y as i32) {
            self.position.y = new_pos.y;
        }
        
        self.bob_phase += 0.2;
    }

    pub fn strafe_right(&mut self, world: &World, delta: f64) {
        let right = Vec2::new(self.direction.y, -self.direction.x);
        let new_pos = self.position + right * (self.move_speed * delta);
        if !world.is_wall(new_pos.x as i32, self.position.y as i32) {
            self.position.x = new_pos.x;
        }
        if !world.is_wall(self.position.x as i32, new_pos.y as i32) {
            self.position.y = new_pos.y;
        }
        
        self.bob_phase += 0.2;
    }

    pub fn rotate(&mut self, angle: f64) {
        let rot_angle = angle * self.rot_speed;
        self.direction = self.direction.rotate(rot_angle);
        self.plane = self.plane.rotate(rot_angle);
    }
    
    pub fn rotate_absolute(&mut self, angle: f64) {
        self.direction = self.direction.rotate(angle);
        self.plane = self.plane.rotate(angle);
    }

    pub fn look_up(&mut self, delta: f64) {
        self.pitch = (self.pitch + delta * 0.05).clamp(-PI / 3.0, PI / 3.0);
    }

    pub fn look_down(&mut self, delta: f64) {
        self.pitch = (self.pitch - delta * 0.05).clamp(-PI / 3.0, PI / 3.0);
    }

    pub fn update(&mut self, _delta_time: f64) {
        self.z_velocity -= 0.02;
        self.z_position += self.z_velocity;
        
        if self.z_position < 0.0 {
            self.z_position = 0.0;
            self.z_velocity = 0.0;
        }
        
        self.z_velocity *= 0.95;
    }

    pub fn get_view_bob(&self) -> f64 {
        (self.bob_phase.sin() * 0.08).clamp(-0.12, 0.12)
    }

    pub fn get_horizon_offset(&self) -> i32 {
        let base_offset = (self.pitch * 150.0) as i32;
        let bob_offset = (self.get_view_bob() * 20.0) as i32;
        let jump_offset = (self.z_position * 50.0) as i32;
        
        base_offset + bob_offset + jump_offset
    }
}
