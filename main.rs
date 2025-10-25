use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Gauge},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};
use rand::Rng;

mod vec2;
mod world;
mod camera;
mod renderer;
mod maze_gen;
mod entities;

use vec2::Vec2;
use world::World;
use camera::Camera;
use renderer::Renderer;
use entities::{Item, ItemType, NPC, NPCType};

const TARGET_FPS: u64 = 60;
const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);

#[derive(Clone, Copy, PartialEq)]
enum Button {
    Forward,
    Backward,
    StrafeLeft,
    StrafeRight,
    RotateLeft,
    RotateRight,
    ResetView,
    NewMaze,
}

struct ButtonState {
    button: Button,
    rect: Rect,
    pressed: bool,
    hover: bool,
    press_time: Option<Instant>,
}

impl ButtonState {
    fn new(button: Button) -> Self {
        ButtonState {
            button,
            rect: Rect::default(),
            pressed: false,
            hover: false,
            press_time: None,
        }
    }

    fn is_inside(&self, x: u16, y: u16) -> bool {
        x >= self.rect.x && x < self.rect.x + self.rect.width &&
        y >= self.rect.y && y < self.rect.y + self.rect.height
    }

    fn get_style(&self) -> Style {
        let now = Instant::now();
        let base_color = match self.button {
            Button::Forward | Button::Backward => Color::Cyan,
            Button::StrafeLeft | Button::StrafeRight => Color::Green,
            Button::RotateLeft | Button::RotateRight => Color::Yellow,
            Button::ResetView => Color::LightBlue,
            Button::NewMaze => Color::Magenta,
        };

        if self.pressed {
            if let Some(press_time) = self.press_time {
                let elapsed = now.duration_since(press_time).as_millis();
                if elapsed < 150 {
                    return Style::default()
                        .fg(Color::White)
                        .bg(base_color)
                        .add_modifier(Modifier::BOLD);
                }
            }
        }

        if self.hover {
            Style::default()
                .fg(base_color)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(base_color)
        }
    }

    fn get_label(&self) -> &str {
        match self.button {
            Button::Forward => "▲ Forward",
            Button::Backward => "▼ Back",
            Button::StrafeLeft => "◄ Left",
            Button::StrafeRight => "► Right",
            Button::RotateLeft => "↺ Turn L",
            Button::RotateRight => "↻ Turn R",
            Button::ResetView => "⊡ Level",
            Button::NewMaze => "🔄 New Maze",
        }
    }
}

struct App {
    camera: Camera,
    world: World,
    renderer: Renderer,
    running: bool,
    fps: f64,
    buttons: Vec<ButtonState>,
    mouse_dragging: bool,
    last_mouse_pos: Option<(u16, u16)>,
    animation_frame: usize,
    health: f64,
    steps: u32,
    items: Vec<Item>,
    npcs: Vec<NPC>,
    coins_collected: u32,
    keys_collected: u32,
    monochrome_mode: bool,
    energy_bar_rect: Option<Rect>,
    // 添加用于跟踪持续按压的字段
    pressed_button: Option<Button>,
    button_press_time: Option<Instant>,
    // 添加全屏视角模式相关字段
    fullscreen_mode: bool,
    minimap_rect: Option<Rect>,
}

impl App {
    fn new() -> Self {
        let world = World::new_random();
        let start_pos = world.get_start_position();
        let camera = Camera::new(Vec2::new(start_pos.0, start_pos.1), Vec2::new(-1.0, 0.0));
        let renderer = Renderer::new();

        let buttons = vec![
            ButtonState::new(Button::Forward),
            ButtonState::new(Button::Backward),
            ButtonState::new(Button::StrafeLeft),
            ButtonState::new(Button::StrafeRight),
            ButtonState::new(Button::RotateLeft),
            ButtonState::new(Button::RotateRight),
            ButtonState::new(Button::ResetView),
            ButtonState::new(Button::NewMaze),
        ];

        let mut items = Vec::new();
        let mut npcs = Vec::new();
        
        for _ in 0..8 {
            let mut rng = rand::thread_rng();
            loop {
                let x = rng.gen_range(5..world.width - 5) as f64;
                let y = rng.gen_range(5..world.height - 5) as f64;
                if !world.is_wall(x as i32, y as i32) {
                    items.push(Item::new(x + 0.5, y + 0.5, ItemType::Coin));
                    break;
                }
            }
        }
        
        for _ in 0..2 {
            let mut rng = rand::thread_rng();
            loop {
                let x = rng.gen_range(5..world.width - 5) as f64;
                let y = rng.gen_range(5..world.height - 5) as f64;
                if !world.is_wall(x as i32, y as i32) {
                    items.push(Item::new(x + 0.5, y + 0.5, ItemType::Key));
                    break;
                }
            }
        }

        for npc_type in [NPCType::Wanderer, NPCType::Guard] {
            let mut rng = rand::thread_rng();
            loop {
                let x = rng.gen_range(5..world.width - 5) as f64;
                let y = rng.gen_range(5..world.height - 5) as f64;
                if !world.is_wall(x as i32, y as i32) {
                    npcs.push(NPC::new(x + 0.5, y + 0.5, npc_type));
                    break;
                }
            }
        }

        App {
            camera,
            world,
            renderer,
            running: true,
            fps: 0.0,
            buttons,
            mouse_dragging: false,
            last_mouse_pos: None,
            animation_frame: 0,
            health: 100.0,
            steps: 0,
            items,
            npcs,
            coins_collected: 0,
            keys_collected: 0,
            monochrome_mode: false,  // 默认彩色模式
            energy_bar_rect: None,
            pressed_button: None,
            button_press_time: None,
            fullscreen_mode: false,
            minimap_rect: None,
        }
    }

    fn regenerate_maze(&mut self) {
        let current_monochrome = self.monochrome_mode;  // 保存当前模式设置
        
        self.world = World::new_random();
        let start_pos = self.world.get_start_position();
        self.camera.position = Vec2::new(start_pos.0, start_pos.1);
        self.steps = 0;
        self.coins_collected = 0;
        self.keys_collected = 0;
        
        self.items.clear();
        self.npcs.clear();
        
        self.monochrome_mode = current_monochrome;  // 恢复模式设置
        self.energy_bar_rect = None;  // 重置energy条矩形
        
        for _ in 0..8 {
            let mut rng = rand::thread_rng();
            loop {
                let x = rng.gen_range(5..self.world.width - 5) as f64;
                let y = rng.gen_range(5..self.world.height - 5) as f64;
                if !self.world.is_wall(x as i32, y as i32) {
                    self.items.push(Item::new(x + 0.5, y + 0.5, ItemType::Coin));
                    break;
                }
            }
        }
        
        for _ in 0..2 {
            let mut rng = rand::thread_rng();
            loop {
                let x = rng.gen_range(5..self.world.width - 5) as f64;
                let y = rng.gen_range(5..self.world.height - 5) as f64;
                if !self.world.is_wall(x as i32, y as i32) {
                    self.items.push(Item::new(x + 0.5, y + 0.5, ItemType::Key));
                    break;
                }
            }
        }

        for npc_type in [NPCType::Wanderer, NPCType::Guard] {
            let mut rng = rand::thread_rng();
            loop {
                let x = rng.gen_range(5..self.world.width - 5) as f64;
                let y = rng.gen_range(5..self.world.height - 5) as f64;
                if !self.world.is_wall(x as i32, y as i32) {
                    self.npcs.push(NPC::new(x + 0.5, y + 0.5, npc_type));
                    break;
                }
            }
        }
    }

    fn execute_button_action(&mut self, button: Button) {
        match button {
            Button::Forward => {
                self.camera.move_forward(&self.world, 1.5);
                self.steps += 1;
                self.check_item_collection();
            }
            Button::Backward => {
                self.camera.move_backward(&self.world, 1.5);
                self.steps += 1;
                self.check_item_collection();
            }
            Button::StrafeLeft => {
                self.camera.strafe_left(&self.world, 1.5);
                self.steps += 1;
                self.check_item_collection();
            }
            Button::StrafeRight => {
                self.camera.strafe_right(&self.world, 1.5);
                self.steps += 1;
                self.check_item_collection();
            }
            Button::RotateLeft => self.camera.rotate(-1.5),
            Button::RotateRight => self.camera.rotate(1.5),
            Button::ResetView => {
                self.camera.pitch = 0.0;
                self.camera.z_position = 0.0;
                self.camera.z_velocity = 0.0;
            }
            Button::NewMaze => self.regenerate_maze(),
        }
    }

    fn check_item_collection(&mut self) {
        let pos = self.camera.position;
        for item in &mut self.items {
            if !item.collected && item.distance_to(pos.x, pos.y) < 0.6 {
                item.collected = true;
                match item.item_type {
                    ItemType::Coin => self.coins_collected += 1,
                    ItemType::Key => self.keys_collected += 1,
                    ItemType::Health => self.health = (self.health + 20.0).min(100.0),
                    _ => {}
                }
            }
        }
    }
    
    fn update_npcs(&mut self) {
        let map = self.world.get_map();
        for npc in &mut self.npcs {
            npc.update(map, 1.0 / 30.0);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('w') | KeyCode::Up => self.execute_button_action(Button::Forward),
                        KeyCode::Char('s') | KeyCode::Down => self.execute_button_action(Button::Backward),
                        KeyCode::Char('a') => self.execute_button_action(Button::StrafeLeft),
                        KeyCode::Char('d') => self.execute_button_action(Button::StrafeRight),
                        KeyCode::Left => self.execute_button_action(Button::RotateLeft),
                        KeyCode::Right => self.execute_button_action(Button::RotateRight),
                        KeyCode::Char('e') => self.camera.look_up(1.0),
                        KeyCode::Char('c') => self.camera.look_down(1.0),
                        KeyCode::Char(' ') => {
                            if self.camera.z_position == 0.0 {
                                self.camera.z_velocity = 0.3;
                            }
                        }
                        KeyCode::Char('r') => self.execute_button_action(Button::NewMaze),
                        KeyCode::Char('m') => self.monochrome_mode = !self.monochrome_mode, // 切换纯色模式
                        KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                        _ => {}
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let mut clicked_button = None;
                            for button in &mut self.buttons {
                                if button.is_inside(mouse.column, mouse.row) {
                                    button.pressed = true;
                                    button.press_time = Some(Instant::now());
                                    clicked_button = Some(button.button);
                                }
                            }
                            
                            // 检查是否点击了energy条
                            if let Some(energy_rect) = self.energy_bar_rect {
                                if mouse.column >= energy_rect.x && mouse.column < energy_rect.x + energy_rect.width &&
                                   mouse.row >= energy_rect.y && mouse.row < energy_rect.y + energy_rect.height {
                                    self.monochrome_mode = !self.monochrome_mode;
                                }
                            }
                            
                            // 检查是否点击了地图区域
                            if let Some(minimap_rect) = self.minimap_rect {
                                if mouse.column >= minimap_rect.x && mouse.column < minimap_rect.x + minimap_rect.width &&
                                   mouse.row >= minimap_rect.y && mouse.row < minimap_rect.y + minimap_rect.height {
                                    self.fullscreen_mode = !self.fullscreen_mode;
                                }
                            }
                            
                            if let Some(btn) = clicked_button {
                                self.execute_button_action(btn);
                                // 记录按压的按钮和时间，用于持续移动
                                self.pressed_button = Some(btn);
                                self.button_press_time = Some(Instant::now());
                            }
                            self.mouse_dragging = true;
                            self.last_mouse_pos = Some((mouse.column, mouse.row));
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            for button in &mut self.buttons {
                                button.pressed = false;
                            }
                            self.mouse_dragging = false;
                            // 清除按压状态
                            self.pressed_button = None;
                            self.button_press_time = None;
                        }
                        MouseEventKind::Drag(MouseButton::Left) => {
                            if self.mouse_dragging {
                                if let Some((last_x, last_y)) = self.last_mouse_pos {
                                    let delta_x = mouse.column as i16 - last_x as i16;
                                    let delta_y = mouse.row as i16 - last_y as i16;
                                    
                                    if delta_x.abs() > 0 {
                                        let rotation = delta_x as f64 * 0.02;
                                        self.camera.rotate_absolute(rotation);
                                    }
                                    
                                    if delta_y.abs() > 0 {
                                        if delta_y < 0 {
                                            self.camera.look_up(delta_y.abs() as f64 * 0.5);
                                        } else {
                                            self.camera.look_down(delta_y as f64 * 0.5);
                                        }
                                    }
                                }
                                self.last_mouse_pos = Some((mouse.column, mouse.row));
                            }
                        }
                        MouseEventKind::Moved => {
                            for button in &mut self.buttons {
                                button.hover = button.is_inside(mouse.column, mouse.row);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        self.animation_frame = (self.animation_frame + 1) % 60;
        self.camera.update(1.0 / 30.0);
        self.update_npcs();
        
        // 处理持续按钮按压
        if let Some(button) = self.pressed_button {
            if let Some(press_time) = self.button_press_time {
                let elapsed = Instant::now().duration_since(press_time);
                // 按下超过300毫秒后开始持续移动，每100毫秒执行一次
                if elapsed.as_millis() > 300 && (elapsed.as_millis() - 300) % 100 < 16 {
                    match button {
                        Button::Forward | Button::Backward | Button::StrafeLeft | Button::StrafeRight => {
                            // 只对移动按钮执行持续移动
                            self.execute_button_action(button);
                        }
                        _ => {} // 其他按钮不执行持续动作
                    }
                }
            }
        }
        
        terminal.draw(|frame| {
            let size = frame.area();
            
            // 根据全屏模式调整布局
            if self.fullscreen_mode {
                // 全屏模式：3D视角占据整个屏幕
                self.renderer.render(frame, size, &self.camera, &self.world, &self.items, &self.npcs, self.monochrome_mode);
            } else {
                // 正常模式：三栏布局
                let main_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                        Constraint::Percentage(20),
                    ])
                    .split(size);

                let left_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(8),
                    ])
                    .split(main_chunks[0]);

                let center_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),
                        Constraint::Length(5),
                    ])
                    .split(main_chunks[1]);

                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ])
                    .split(main_chunks[2]);

                self.renderer.render(frame, center_chunks[0], &self.camera, &self.world, &self.items, &self.npcs, self.monochrome_mode);

                self.buttons[0].rect = left_chunks[0];
                self.buttons[1].rect = left_chunks[1];
                self.buttons[2].rect = left_chunks[2];
                self.buttons[3].rect = left_chunks[3];

                for i in 0..4 {
                    let button = &self.buttons[i];
                    let label = button.get_label();
                    let style = button.get_style();
                    
                    let border_type = if button.pressed {
                        BorderType::Double
                    } else if button.hover {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    };

                    let btn = Paragraph::new(label)
                        .style(style)
                        .alignment(Alignment::Center)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .border_type(border_type));
                    frame.render_widget(btn, left_chunks[i]);
                }

                self.buttons[4].rect = Rect {
                    x: left_chunks[5].x + 1,
                    y: left_chunks[5].y + 1,
                    width: left_chunks[5].width / 2 - 1,
                    height: 3,
                };
                self.buttons[5].rect = Rect {
                    x: left_chunks[5].x + left_chunks[5].width / 2,
                    y: left_chunks[5].y + 1,
                    width: left_chunks[5].width / 2 - 1,
                    height: 3,
                };
                self.buttons[6].rect = Rect {
                    x: left_chunks[5].x + 1,
                    y: left_chunks[5].y + 4,
                    width: left_chunks[5].width / 2 - 1,
                    height: 3,
                };
                self.buttons[7].rect = Rect {
                    x: left_chunks[5].x + left_chunks[5].width / 2,
                    y: left_chunks[5].y + 4,
                    width: left_chunks[5].width / 2 - 1,
                    height: 3,
                };

                for i in 4..8 {
                    let button = &self.buttons[i];
                    let label = button.get_label();
                    let style = button.get_style();
                    
                    let border_type = if button.pressed {
                        BorderType::Double
                    } else if button.hover {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    };

                    let btn = Paragraph::new(label)
                        .style(style)
                        .alignment(Alignment::Center)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .border_type(border_type));
                    frame.render_widget(btn, button.rect);
                }

                let controls_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("🎮 Controls");
                frame.render_widget(controls_block, left_chunks[5]);

                let pos = self.camera.position;
                let dir = self.camera.direction;
                
                let wall_dist = self.get_nearest_wall_distance();
                let proximity_warning = if wall_dist < 1.5 {
                    Span::styled("⚠ WALL! ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK))
                } else if wall_dist < 3.0 {
                    Span::styled("⚠ Close ", Style::default().fg(Color::Yellow))
                } else {
                    Span::styled("✓ Clear ", Style::default().fg(Color::Green))
                };

                let animation_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let anim_char = animation_chars[self.animation_frame / 6 % animation_chars.len()];

                let pitch_degrees = (self.camera.pitch * 180.0 / std::f64::consts::PI) as i32;
                let pitch_indicator = if self.camera.z_position > 0.1 {
                    Span::styled("↑ JUMP ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else if pitch_degrees > 10 {
                    Span::styled(format!("↗ +{}° ", pitch_degrees), Style::default().fg(Color::Cyan))
                } else if pitch_degrees < -10 {
                    Span::styled(format!("↘ {}° ", pitch_degrees), Style::default().fg(Color::Blue))
                } else {
                    Span::styled("→ Level ", Style::default().fg(Color::Green))
                };

                let info_lines = vec![
                    Line::from(vec![
                        Span::styled(format!("{} ", anim_char), Style::default().fg(Color::Cyan)),
                        Span::styled("Position: ", Style::default().fg(Color::Gray)),
                        Span::raw(format!("({:.1}, {:.1})", pos.x, pos.y)),
                    ]),
                    Line::from(vec![
                        Span::styled("Direction: ", Style::default().fg(Color::Gray)),
                        Span::raw(format!("({:.2}, {:.2})", dir.x, dir.y)),
                    ]),
                    Line::from(vec![
                        pitch_indicator,
                    ]),
                    Line::from(vec![
                        proximity_warning,
                        Span::raw(format!("Dist: {:.1}", wall_dist)),
                    ]),
                    Line::from(vec![
                        Span::styled(format!("Steps: {}", self.steps), Style::default().fg(Color::Magenta)),
                        Span::raw("  "),
                        Span::styled(format!("FPS: {:.0}", self.fps), Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(vec![
                        Span::styled("Mode: ", Style::default().fg(Color::Gray)),
                        Span::styled(if self.monochrome_mode { "MONOCHROME" } else { "COLOR" }, 
                            Style::default().fg(if self.monochrome_mode { Color::White } else { Color::Cyan }).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("◆", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::raw(format!(":{} ", self.coins_collected)),
                        Span::styled("🔑", Style::default().fg(Color::Cyan)),
                        Span::raw(format!(":{}", self.keys_collected)),
                    ]),
                ];

                let info = Paragraph::new(info_lines)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("📊 Status"))
                    .alignment(Alignment::Left);
                frame.render_widget(info, center_chunks[1]);

                self.renderer.render_minimap(frame, right_chunks[0], &self.camera, &self.world, &self.items, &self.npcs, self.monochrome_mode);

                let help_text = vec![
                    Line::from(vec![
                        Span::styled("🖱️ Mouse", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from("• Click buttons"),
                    Line::from("• Drag X: Rotate"),
                    Line::from("• Drag Y: Look"),
                    Line::from("• Click Map: Fullscreen"),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("⌨️ Keyboard", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from("WASD: Move"),
                    Line::from("←→: Rotate"),
                    Line::from("E/C: Look up/down"),
                    Line::from("Space: Jump"),
                    Line::from("R: New maze"),
                    Line::from("Q: Quit"),
                    Line::from("M: Color/Mono"),
                ];

                let help = Paragraph::new(help_text)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("ℹ️ Help"))
                    .alignment(Alignment::Left);
                frame.render_widget(help, right_chunks[1]);

                let health_gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Energy"))
                    .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
                    .percent(self.health as u16);
                frame.render_widget(health_gauge, left_chunks[4]);
                
                // 保存energy条的矩形位置，用于鼠标点击检测
                self.energy_bar_rect = Some(left_chunks[4]);
                // 存储地图区域坐标，用于点击检测
                self.minimap_rect = Some(right_chunks[0]);
            }
        })?;
        Ok(())
    }

    fn get_nearest_wall_distance(&self) -> f64 {
        let pos = self.camera.position;
        let dir = self.camera.direction;
        
        for dist in 1..20 {
            let check_x = (pos.x + dir.x * dist as f64) as i32;
            let check_y = (pos.y + dir.y * dist as f64) as i32;
            if self.world.is_wall(check_x, check_y) {
                return dist as f64;
            }
        }
        20.0
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut frame_count = 0;
    let mut fps_timer = Instant::now();

    terminal.clear()?;

    while app.running {
        let frame_start = Instant::now();

        app.handle_events()?;
        app.render(&mut terminal)?;

        frame_count += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            app.fps = frame_count as f64 / fps_timer.elapsed().as_secs_f64();
            frame_count = 0;
            fps_timer = Instant::now();
        }

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - elapsed);
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
