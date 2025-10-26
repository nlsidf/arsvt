use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph},
    Frame,
};

use crate::camera::Camera;
use crate::world::{World, WallType};
use crate::entities::{Item, NPC};

pub struct Renderer {
    buffer: Vec<Vec<char>>,
    color_buffer: Vec<Vec<Color>>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            buffer: Vec::new(),
            color_buffer: Vec::new(),
        }
    }

    fn resize_buffers(&mut self, width: usize, height: usize) {
        if self.buffer.len() != height || (self.buffer.len() > 0 && self.buffer[0].len() != width) {
            self.buffer = vec![vec![' '; width]; height];
            self.color_buffer = vec![vec![Color::Black; width]; height];
        }
    }

    fn clear(&mut self, width: usize, height: usize) {
        self.resize_buffers(width, height);
        
        for y in 0..height {
            for x in 0..width {
                if y < height / 3 {
                    let ceiling_depth = y as f64 / (height as f64 / 3.0);
                    let ceiling_brightness = (0.1 + ceiling_depth * 0.15) as u8;
                    self.buffer[y][x] = match ceiling_brightness {
                        0..=5 => ' ',
                        6..=10 => '·',
                        11..=15 => '░',
                        _ => '▒',
                    };
                    self.color_buffer[y][x] = Color::Rgb(
                        20 + ceiling_brightness,
                        20 + ceiling_brightness,
                        40 + ceiling_brightness * 2
                    );
                } else if y >= height * 2 / 3 {
                    let floor_y = y - height * 2 / 3;
                    let floor_depth = (height / 3) as f64 / (floor_y as f64 + 1.0);
                    let floor_brightness = (1.0 / (1.0 + floor_depth * 0.2)).clamp(0.0, 1.0);
                    
                    let pattern = (x / 2 + floor_y / 2) % 2;
                    let base_char = if pattern == 0 { '▓' } else { '▒' };
                    
                    self.buffer[y][x] = if floor_brightness < 0.2 {
                        ' '
                    } else if floor_brightness < 0.4 {
                        '·'
                    } else if floor_brightness < 0.6 {
                        '░'
                    } else {
                        base_char
                    };
                    
                    self.color_buffer[y][x] = Color::Rgb(
                        (70.0 * floor_brightness) as u8,
                        (55.0 * floor_brightness) as u8,
                        (35.0 * floor_brightness) as u8
                    );
                } else {
                    self.buffer[y][x] = ' ';
                    self.color_buffer[y][x] = Color::Black;
                }
            }
        }
    }

    fn get_wall_color(&self, wall_type: WallType, brightness: f64, distance: f64) -> Color {
        let base = match wall_type {
            WallType::Red => (255, 80, 80),
            WallType::Green => (80, 255, 80),
            WallType::Blue => (100, 160, 255),
            WallType::White => (255, 255, 255),
            WallType::Yellow => (255, 255, 80),
            WallType::Empty => (128, 128, 128),
        };
        
        let fog_factor = (1.0 / (1.0 + distance * 0.08)).clamp(0.0, 1.0);
        let bright = (brightness * fog_factor).clamp(0.1, 1.0);
        
        let fog_color = (30, 30, 60);
        
        Color::Rgb(
            ((base.0 as f64 * bright) + (fog_color.0 as f64 * (1.0 - fog_factor))) as u8,
            ((base.1 as f64 * bright) + (fog_color.1 as f64 * (1.0 - fog_factor))) as u8,
            ((base.2 as f64 * bright) + (fog_color.2 as f64 * (1.0 - fog_factor))) as u8,
        )
    }

    fn get_char(&self, distance: f64, side: bool, wall_x: f64, y_ratio: f64) -> char {
        let brightness = 1.0 / (1.0 + distance * distance * 0.025);
        let adjusted = if side { brightness * 0.7 } else { brightness };
        
        let brick_x = (wall_x * 4.0) as usize % 4;
        let brick_y = (y_ratio * 6.0) as usize % 6;
        
        let is_mortar_h = brick_y == 0 || brick_y == 3;
        let is_mortar_v = brick_x == 0;
        let is_edge = y_ratio < 0.05 || y_ratio > 0.95;
        
        if adjusted > 0.75 {
            if is_edge {
                '═'
            } else if is_mortar_h || is_mortar_v {
                '░'
            } else {
                '█'
            }
        } else if adjusted > 0.55 {
            if is_mortar_h || is_mortar_v {
                '░'
            } else {
                '▓'
            }
        } else if adjusted > 0.35 {
            if is_mortar_h {
                '·'
            } else {
                '▒'
            }
        } else if adjusted > 0.20 {
            '░'
        } else {
            '·'
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, camera: &Camera, world: &World, items: &[Item], npcs: &[NPC], monochrome_mode: bool) {
        let width = area.width.saturating_sub(2) as usize;
        let height = area.height.saturating_sub(2) as usize;
        
        if width == 0 || height == 0 {
            return;
        }
        
        self.clear(width, height);

        let pos = camera.position;
        let dir = camera.direction;
        let plane = camera.plane;
        let horizon_offset = camera.get_horizon_offset();

        for x in 0..width {
            let camera_x = 2.0 * x as f64 / width as f64 - 1.0;
            let ray_dir_x = dir.x + plane.x * camera_x;
            let ray_dir_y = dir.y + plane.y * camera_x;

            let mut map_x = pos.x as i32;
            let mut map_y = pos.y as i32;

            let delta_dist_x = if ray_dir_x.abs() < 1e-10 {
                1e30
            } else {
                (1.0 / ray_dir_x).abs()
            };
            
            let delta_dist_y = if ray_dir_y.abs() < 1e-10 {
                1e30
            } else {
                (1.0 / ray_dir_y).abs()
            };

            let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
                (-1, (pos.x - map_x as f64) * delta_dist_x)
            } else {
                (1, (map_x as f64 + 1.0 - pos.x) * delta_dist_x)
            };

            let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
                (-1, (pos.y - map_y as f64) * delta_dist_y)
            } else {
                (1, (map_y as f64 + 1.0 - pos.y) * delta_dist_y)
            };

            let mut hit = false;
            let mut side = false;
            let mut iterations = 0;

            while !hit && iterations < 100 {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = false;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = true;
                }

                if world.is_wall(map_x, map_y) {
                    hit = true;
                }
                iterations += 1;
            }

            if !hit {
                continue;
            }

            let perp_wall_dist = if !side {
                (side_dist_x - delta_dist_x).max(0.01)
            } else {
                (side_dist_y - delta_dist_y).max(0.01)
            };

            let wall_x = if !side {
                pos.y + perp_wall_dist * ray_dir_y
            } else {
                pos.x + perp_wall_dist * ray_dir_x
            };
            let wall_x = wall_x - wall_x.floor();

            let line_height = ((height as f64 / perp_wall_dist) as usize).min(height * 4);

            let draw_start_base = (height / 2).saturating_sub(line_height / 2);
            let draw_end_base = ((height / 2) + (line_height / 2)).min(height);
            
            let draw_start = ((draw_start_base as i32 + horizon_offset).max(0) as usize).min(height);
            let draw_end = ((draw_end_base as i32 + horizon_offset).max(0) as usize).min(height);

            let wall_type = world.get(map_x, map_y);
            let brightness = 1.0 / (1.0 + perp_wall_dist * perp_wall_dist * 0.03);
            let adjusted_brightness = if side { brightness * 0.65 } else { brightness };

            for y in draw_start..draw_end {
                if y < height && x < width {
                    let y_ratio = (y as f64 - draw_start as f64) / (draw_end - draw_start).max(1) as f64;
                    let ch = self.get_char(perp_wall_dist, side, wall_x, y_ratio);
                    let color = if monochrome_mode {
                        // 纯色模式：所有物体都使用白色
                        let brightness = adjusted_brightness.clamp(0.2, 1.0);
                        Color::Rgb(
                            (255.0 * brightness) as u8,
                            (255.0 * brightness) as u8,
                            (255.0 * brightness) as u8
                        )
                    } else {
                        self.get_wall_color(wall_type, adjusted_brightness, perp_wall_dist)
                    };
                    self.buffer[y][x] = ch;
                    self.color_buffer[y][x] = color;
                }
            }
        }

        let mut sprite_order: Vec<(usize, f64, String, Color)> = Vec::new();
        
        for item in items {
            if item.collected {
                continue;
            }
            let sprite_x = item.x - pos.x;
            let sprite_y = item.y - pos.y;
            
            let inv_det = 1.0 / (plane.x * dir.y - dir.x * plane.y);
            let transform_x = inv_det * (dir.y * sprite_x - dir.x * sprite_y);
            let transform_y = inv_det * (-plane.y * sprite_x + plane.x * sprite_y);
            
            if transform_y > 0.1 && transform_y < 20.0 {
                let sprite_screen_x = ((width as f64 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
                if sprite_screen_x > 0 && sprite_screen_x < width as i32 {
                    let icon = match item.item_type {
                        crate::entities::ItemType::Coin => "◆",
                        crate::entities::ItemType::Key => "🔑",
                        crate::entities::ItemType::Health => "❤",
                        crate::entities::ItemType::Exit => "🚪",
                    };
                    let color = if monochrome_mode {
                        // 纯色模式：所有物品都使用白色
                        Color::White
                    } else {
                        match item.item_type {
                            crate::entities::ItemType::Coin => Color::Yellow,
                            crate::entities::ItemType::Key => Color::Cyan,
                            crate::entities::ItemType::Health => Color::Red,
                            crate::entities::ItemType::Exit => Color::Green,
                        }
                    };
                    sprite_order.push((sprite_screen_x as usize, transform_y, icon.to_string(), color));
                }
            }
        }
        
        for npc in npcs {
            let sprite_x = npc.x - pos.x;
            let sprite_y = npc.y - pos.y;
            
            let inv_det = 1.0 / (plane.x * dir.y - dir.x * plane.y);
            let transform_x = inv_det * (dir.y * sprite_x - dir.x * sprite_y);
            let transform_y = inv_det * (-plane.y * sprite_x + plane.x * sprite_y);
            
            if transform_y > 0.1 && transform_y < 20.0 {
                let sprite_screen_x = ((width as f64 / 2.0) * (1.0 + transform_x / transform_y)) as i32;
                if sprite_screen_x > 0 && sprite_screen_x < width as i32 {
                    let icon = match npc.npc_type {
                        crate::entities::NPCType::Wanderer => "T^T",
                        crate::entities::NPCType::Guard => "(^.^)",
                    };
                    let color = if monochrome_mode {
                        // 纯色模式：所有NPC都使用白色
                        Color::White
                    } else {
                        match npc.npc_type {
                            crate::entities::NPCType::Wanderer => Color::LightGreen,
                            crate::entities::NPCType::Guard => Color::LightRed,
                        }
                    };
                    sprite_order.push((sprite_screen_x as usize, transform_y, icon.to_string(), color));
                }
            }
        }
        
        sprite_order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        for (screen_x, depth, icon, color) in sprite_order {
            if screen_x < width {
                let sprite_height = ((height as f64 / depth) as usize).min(height / 2);
                let draw_y = ((height / 2).saturating_sub(sprite_height / 4) as isize + horizon_offset.max(-20).min(20) as isize).max(0) as usize;
                
                if draw_y < height {
                    // 绘制多字符图标，每个字符占据一个屏幕位置
                    for (i, ch) in icon.chars().enumerate() {
                        let current_x = screen_x + i;
                        if current_x < width {
                            self.buffer[draw_y][current_x] = ch;
                            self.color_buffer[draw_y][current_x] = color;
                        }
                    }
                }
            }
        }

        let lines: Vec<Line> = self.buffer.iter().enumerate().map(|(y, row)| {
            let spans: Vec<Span> = row.iter().enumerate().map(|(x, &ch)| {
                Span::styled(
                    ch.to_string(), 
                    Style::default().fg(self.color_buffer[y][x])
                )
            }).collect();
            Line::from(spans)
        }).collect();

        let paragraph = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title(vec![
                    Span::styled("═══ ", Style::default().fg(Color::DarkGray)),
                    Span::styled("🎮 3D VIEW ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled("═══", Style::default().fg(Color::DarkGray)),
                ]));
        frame.render_widget(paragraph, area);
    }

    pub fn render_minimap(&self, frame: &mut Frame, area: Rect, camera: &Camera, world: &World, items: &[Item], npcs: &[NPC], monochrome_mode: bool) {
        let map = world.get_map();
        let view_size = 24;
        
        let center_x = camera.position.x as usize;
        let center_y = camera.position.y as usize;
        
        let start_x = center_x.saturating_sub(view_size / 2);
        let start_y = center_y.saturating_sub(view_size / 2);
        
        let mut lines: Vec<Line> = Vec::new();
        
        for dy in 0..view_size.min(area.height.saturating_sub(2) as usize) {
            let mut spans = Vec::new();
            for dx in 0..view_size.min(area.width.saturating_sub(2) as usize) {
                let map_x = start_x + dx;
                let map_y = start_y + dy;
                
                let player_dx = map_x as i32 - center_x as i32;
                let player_dy = map_y as i32 - center_y as i32;
                let dist_sq = player_dx * player_dx + player_dy * player_dy;
                
                if dist_sq == 0 {
                    let dir_angle = camera.direction.y.atan2(camera.direction.x).to_degrees();
                    let dir_char = match ((dir_angle + 360.0) % 360.0) as i32 {
                        0..=22 | 338..=360 => '→',
                        23..=67 => '↗',
                        68..=112 => '↑',
                        113..=157 => '↖',
                        158..=202 => '←',
                        203..=247 => '↙',
                        248..=292 => '↓',
                        _ => '↘',
                    };
                    spans.push(Span::styled(
                        dir_char.to_string(), 
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    ));
                } else if dist_sq <= 4 {
                    spans.push(Span::styled("◉", Style::default().fg(Color::Yellow)));
                } else if map_x < world.width && map_y < world.height {
                    let mut found_item = false;
                    for item in items {
                        if !item.collected && (item.x as usize) == map_x && (item.y as usize) == map_y {
                            let icon = match item.item_type {
                                crate::entities::ItemType::Coin => '◆',
                                crate::entities::ItemType::Key => '🔑',
                                crate::entities::ItemType::Health => '❤',
                                crate::entities::ItemType::Exit => '🚪',
                            };
                            let color = match item.item_type {
                                crate::entities::ItemType::Coin => Color::Yellow,
                                crate::entities::ItemType::Key => Color::Cyan,
                                crate::entities::ItemType::Health => Color::Red,
                                crate::entities::ItemType::Exit => Color::Green,
                            };
                            spans.push(Span::styled(icon.to_string(), Style::default().fg(color)));
                            found_item = true;
                            break;
                        }
                    }
                    
                    if !found_item {
                        for npc in npcs {
                            if (npc.x as usize) == map_x && (npc.y as usize) == map_y {
                                let icon = match npc.npc_type {
                                    crate::entities::NPCType::Wanderer => "T^T",
                                    crate::entities::NPCType::Guard => "(^.^)",
                                };
                                let color = match npc.npc_type {
                                    crate::entities::NPCType::Wanderer => Color::LightGreen,
                                    crate::entities::NPCType::Guard => Color::LightRed,
                                };
                                spans.push(Span::styled(icon.to_string(), Style::default().fg(color)));
                                found_item = true;
                                break;
                            }
                        }
                    }
                    
                    if !found_item {
                        if map[map_x][map_y] != WallType::Empty {
                            let wall_color = if monochrome_mode {
                                Color::White
                            } else {
                                match map[map_x][map_y] {
                                    WallType::Red => Color::Red,
                                    WallType::Green => Color::Green,
                                    WallType::Blue => Color::Blue,
                                    WallType::White => Color::White,
                                    WallType::Yellow => Color::Yellow,
                                    _ => Color::Gray,
                                }
                            };
                            spans.push(Span::styled("█", Style::default().fg(wall_color)));
                        } else {
                            let is_visited = dist_sq < 100;
                            if is_visited {
                                spans.push(Span::styled("·", Style::default().fg(Color::DarkGray)));
                            } else {
                                spans.push(Span::styled(" ", Style::default()));
                            }
                        }
                    }
                } else {
                    spans.push(Span::raw(" "));
                }
            }
            lines.push(Line::from(spans));
        }
        
        let minimap = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(vec![
                    Span::styled("🗺️ ", Style::default().fg(Color::Yellow)),
                    Span::styled("Minimap", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
        frame.render_widget(minimap, area);
    }
}
