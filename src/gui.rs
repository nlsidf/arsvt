use egui::{Context, CentralPanel, SidePanel, TopBottomPanel, Frame, Color32, Vec2, RichText, FontId, Response, Painter};
use std::f64::consts::PI;
use rand::Rng;

// 导入现有的模块
use crate::vec2::Vec2 as Vec2D;
use crate::camera::Camera;
use crate::world::{World, WallType};
use crate::renderer::Renderer;
use crate::entities::{Item, ItemType, NPC, NPCType};

// 按钮类型枚举
#[derive(Clone, Copy, PartialEq, Debug)]
enum ButtonType {
    Forward,
    Backward,
    StrafeLeft,
    StrafeRight,
    RotateLeft,
    RotateRight,
    ResetView,
    NewMaze,
}

// GUI应用结构体
pub struct GUIApp {
    camera: Camera,
    world: World,
    renderer: Renderer,
    items: Vec<Item>,
    npcs: Vec<NPC>,
    health: f64,
    steps: u32,
    coins_collected: u32,
    keys_collected: u32,
    monochrome_mode: bool,
    fullscreen_mode: bool,
    // 按钮状态
    button_hover: Option<ButtonType>,
    button_pressed: Option<ButtonType>,
    // 鼠标拖拽状态
    mouse_dragging: bool,
    last_mouse_pos: Option<egui::Pos2>,
    // 动画帧
    animation_frame: usize,
    // 按钮持续按压
    pressed_button: Option<ButtonType>,
    button_press_time: Option<std::time::Instant>,
}

impl GUIApp {
    pub fn new() -> Self {
        let world = World::new_random();
        let start_pos = world.get_start_position();
        let camera = Camera::new(Vec2D::new(start_pos.0, start_pos.1), Vec2D::new(-1.0, 0.0));
        let renderer = Renderer::new();
        
        // 初始化物品和NPC（这部分逻辑从main.rs中移过来）
        let mut items = Vec::new();
        let mut npcs = Vec::new();
        
        // 初始化物品
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

        // 初始化NPC
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
        
        Self {
            camera,
            world,
            renderer,
            items,
            npcs,
            health: 100.0,
            steps: 0,
            coins_collected: 0,
            keys_collected: 0,
            monochrome_mode: false,
            fullscreen_mode: false,
            button_hover: None,
            button_pressed: None,
            mouse_dragging: false,
            last_mouse_pos: None,
            animation_frame: 0,
            pressed_button: None,
            button_press_time: None,
        }
    }
    
    // 主更新函数
    pub fn update(&mut self, ctx: &Context) {
        // 更新动画帧
        self.animation_frame = (self.animation_frame + 1) % 60;
        
        // 更新相机
        self.camera.update(1.0 / 60.0);
        
        // 更新NPC
        self.update_npcs();
        
        // 处理持续按钮按压
        self.handle_button_repeat();
        
        // 渲染UI
        self.render_ui(ctx);
    }
    
    // 更新NPC
    fn update_npcs(&mut self) {
        let map = self.world.get_map();
        for npc in &mut self.npcs {
            npc.update(map, 1.0 / 60.0);
        }
    }
    
    // 处理按钮重复按压
    fn handle_button_repeat(&mut self) {
        if let Some(button) = self.pressed_button {
            if let Some(press_time) = self.button_press_time {
                let elapsed = std::time::Instant::now().duration_since(press_time);
                // 按下超过300毫秒后开始持续移动，每100毫秒执行一次
                if elapsed.as_millis() > 300 && (elapsed.as_millis() - 300) % 100 < 16 {
                    match button {
                        ButtonType::Forward | ButtonType::Backward | ButtonType::StrafeLeft | ButtonType::StrafeRight => {
                            // 只对移动按钮执行持续移动
                            self.execute_button_action(button);
                        }
                        _ => {} // 其他按钮不执行持续动作
                    }
                }
            }
        }
    }
    
    // 执行按钮动作
    fn execute_button_action(&mut self, button: ButtonType) {
        match button {
            ButtonType::Forward => {
                self.move_forward();
            }
            ButtonType::Backward => {
                self.move_backward();
            }
            ButtonType::StrafeLeft => {
                self.strafe_left();
            }
            ButtonType::StrafeRight => {
                self.strafe_right();
            }
            ButtonType::RotateLeft => {
                self.rotate_left();
            }
            ButtonType::RotateRight => {
                self.rotate_right();
            }
            ButtonType::ResetView => {
                self.reset_view();
            }
            ButtonType::NewMaze => {
                self.new_maze();
            }
        }
    }
    
    // 渲染UI
    fn render_ui(&mut self, ctx: &Context) {
        // 顶部状态栏
        TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new("ASCII Raycasting 3D Maze").size(18.0).color(Color32::GOLD));
                ui.separator();
                ui.label(format!("Steps: {}", self.steps));
                ui.label(format!("FPS: {:.1}", ctx.fps()));
                ui.separator();
                ui.label(format!("◆: {}  🔑: {}", self.coins_collected, self.keys_collected));
            });
        });
        
        // 左侧面板 - 控制区
        SidePanel::left("control_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Controls");
                ui.separator();
                
                // 移动按钮
                if self.button(ButtonType::Forward, "▲ Forward", ui).clicked() {
                    self.move_forward();
                }
                if self.button(ButtonType::Backward, "▼ Backward", ui).clicked() {
                    self.move_backward();
                }
                if self.button(ButtonType::StrafeLeft, "◄ Strafe Left", ui).clicked() {
                    self.strafe_left();
                }
                if self.button(ButtonType::StrafeRight, "► Strafe Right", ui).clicked() {
                    self.strafe_right();
                }
                
                ui.separator();
                
                // 旋转按钮
                if self.button(ButtonType::RotateLeft, "↺ Rotate Left", ui).clicked() {
                    self.rotate_left();
                }
                if self.button(ButtonType::RotateRight, "↻ Rotate Right", ui).clicked() {
                    self.rotate_right();
                }
                
                ui.separator();
                
                // 其他控制按钮
                if self.button(ButtonType::ResetView, "⊡ Reset View", ui).clicked() {
                    self.reset_view();
                }
                if self.button(ButtonType::NewMaze, "🔄 New Maze", ui).clicked() {
                    self.new_maze();
                }
                
                ui.separator();
                
                // 能量条
                ui.label("Energy:");
                let health_bar = egui::ProgressBar::new(self.health as f32 / 100.0)
                    .show_percentage()
                    .animate(true);
                ui.add(health_bar);
                
                ui.separator();
                
                // 模式切换
                ui.checkbox(&mut self.monochrome_mode, "Monochrome Mode");
                ui.checkbox(&mut self.fullscreen_mode, "Fullscreen Mode");
            });
        });
        
        // 右侧面板 - 小地图和帮助
        SidePanel::right("info_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Minimap");
                // 这里需要实现小地图绘制
                ui.separator();
                
                ui.heading("Help");
                ui.label("WASD: Move");
                ui.label("Arrow Keys: Rotate");
                ui.label("E/C: Look up/down");
                ui.label("Space: Jump");
                ui.label("R: New maze");
                ui.label("M: Color/Mono");
                ui.label("F: Fullscreen");
            });
        });
        
        // 中央面板 - 3D视图
        CentralPanel::default().show(ctx, |ui| {
            // 处理鼠标输入
            self.handle_mouse_input(ui);
            
            // 获取绘制区域
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), ui.available_height()), egui::Sense::hover());
            
            // 使用Renderer渲染3D视图到缓冲区
            self.renderer.render_to_buffer(
                rect.width() as usize, 
                rect.height() as usize, 
                &self.camera, 
                &self.world, 
                &self.items, 
                &self.npcs, 
                self.monochrome_mode
            );
            
            // 将缓冲区内容绘制到egui中
            self.draw_render_buffer(ui.painter(), rect);
        });
    }
    
    // 自定义按钮组件
    fn button(&mut self, button_type: ButtonType, label: &str, ui: &mut egui::Ui) -> Response {
        let response = ui.button(label);
        
        // 更新按钮状态
        if response.hovered() {
            self.button_hover = Some(button_type);
        } else if self.button_hover == Some(button_type) {
            self.button_hover = None;
        }
        
        if response.clicked() {
            self.button_pressed = Some(button_type);
            // 记录按压的按钮和时间，用于持续移动
            self.pressed_button = Some(button_type);
            self.button_press_time = Some(std::time::Instant::now());
        }
        
        response
    }
    
    // 绘制渲染缓冲区
    fn draw_render_buffer(&self, painter: &Painter, rect: egui::Rect) {
        // 这里需要实现将Renderer的缓冲区内容绘制到egui中
        // 暂时绘制一个简单的占位符
        painter.rect_filled(rect, 0.0, Color32::BLACK);
        
        // 绘制网格线表示3D视图区域
        let width = rect.width();
        let height = rect.height();
        
        // 绘制垂直线
        for i in 0..20 {
            let x = rect.left() + i as f32 * width / 20.0;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, Color32::DARK_GRAY)
            );
        }
        
        // 绘制水平线
        for i in 0..15 {
            let y = rect.top() + i as f32 * height / 15.0;
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, Color32::DARK_GRAY)
            );
        }
        
        // 绘制中心文本
        painter.text(
            rect.center(), 
            egui::Align2::CENTER_CENTER, 
            "3D Rendering Area", 
            FontId::proportional(20.0), 
            Color32::WHITE
        );
        
        // 绘制一些示例文本表示3D场景
        painter.text(
            egui::pos2(rect.center().x, rect.center().y - 40.0), 
            egui::Align2::CENTER_CENTER, 
            "████████████████████", 
            FontId::monospace(14.0), 
            Color32::DARK_GRAY
        );
        
        painter.text(
            egui::pos2(rect.center().x, rect.center().y - 20.0), 
            egui::Align2::CENTER_CENTER, 
            "████████████████████", 
            FontId::monospace(14.0), 
            Color32::GRAY
        );
        
        painter.text(
            egui::pos2(rect.center().x, rect.center().y), 
            egui::Align2::CENTER_CENTER, 
            "████████████████████", 
            FontId::monospace(14.0), 
            Color32::WHITE
        );
        
        painter.text(
            egui::pos2(rect.center().x, rect.center().y + 20.0), 
            egui::Align2::CENTER_CENTER, 
            "████████████████████", 
            FontId::monospace(14.0), 
            Color32::GRAY
        );
        
        painter.text(
            egui::pos2(rect.center().x, rect.center().y + 40.0), 
            egui::Align2::CENTER_CENTER, 
            "████████████████████", 
            FontId::monospace(14.0), 
            Color32::DARK_GRAY
        );
    }
    
    // 处理鼠标输入
    fn handle_mouse_input(&mut self, ui: &mut egui::Ui) {
        let response = ui.interact(ui.max_rect(), egui::Id::new("3d_view"), egui::Sense::click_and_drag());
        
        // 处理鼠标拖拽
        if response.drag_started() {
            self.mouse_dragging = true;
            self.last_mouse_pos = Some(response.interact_pointer_pos().unwrap_or_default());
        }
        
        if response.drag_stopped() {
            self.mouse_dragging = false;
            // 清除按压状态
            self.pressed_button = None;
            self.button_press_time = None;
        }
        
        if self.mouse_dragging {
            if let Some(current_pos) = response.interact_pointer_pos() {
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta = current_pos - last_pos;
                    
                    // 水平拖动旋转视角
                    if delta.x.abs() > 0.0 {
                        let rotation = delta.x as f64 * 0.002;
                        self.camera.rotate_absolute(rotation);
                    }
                    
                    // 垂直拖动上下看
                    if delta.y.abs() > 0.0 {
                        if delta.y < 0.0 {
                            self.camera.look_up((-delta.y) as f64 * 0.005);
                        } else {
                            self.camera.look_down(delta.y as f64 * 0.005);
                        }
                    }
                }
                self.last_mouse_pos = Some(current_pos);
            }
        }
    }
    
    // 控制函数
    fn move_forward(&mut self) {
        self.camera.move_forward(&self.world, 1.5);
        self.steps += 1;
        self.check_item_collection();
    }
    
    fn move_backward(&mut self) {
        self.camera.move_backward(&self.world, 1.5);
        self.steps += 1;
        self.check_item_collection();
    }
    
    fn strafe_left(&mut self) {
        self.camera.strafe_left(&self.world, 1.5);
        self.steps += 1;
        self.check_item_collection();
    }
    
    fn strafe_right(&mut self) {
        self.camera.strafe_right(&self.world, 1.5);
        self.steps += 1;
        self.check_item_collection();
    }
    
    fn rotate_left(&mut self) {
        self.camera.rotate(-1.5);
    }
    
    fn rotate_right(&mut self) {
        self.camera.rotate(1.5);
    }
    
    fn reset_view(&mut self) {
        self.camera.pitch = 0.0;
        self.camera.z_position = 0.0;
        self.camera.z_velocity = 0.0;
    }
    
    fn new_maze(&mut self) {
        let current_monochrome = self.monochrome_mode;  // 保存当前模式设置
        
        // 重新生成迷宫的逻辑
        self.world = World::new_random();
        let start_pos = self.world.get_start_position();
        self.camera.position = Vec2D::new(start_pos.0, start_pos.1);
        self.steps = 0;
        self.coins_collected = 0;
        self.keys_collected = 0;
        
        // 重新初始化物品和NPC
        self.items.clear();
        self.npcs.clear();
        
        self.monochrome_mode = current_monochrome;  // 恢复模式设置
        
        // 重新添加物品
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

        // 重新添加NPC
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
}