use egui_winit::egui;
use std::time::Instant;
use rand::Rng;

mod vec2;
mod world;
mod camera;
mod renderer;
mod maze_gen;
mod entities;
mod gui;

use vec2::Vec2;
use world::World;
use camera::Camera;
use renderer::Renderer;
use entities::{Item, ItemType, NPC, NPCType};
use gui::GUIApp;

// 添加eframe依赖
use eframe::egui as egui_lib;

struct App {
    gui_app: GUIApp,
    running: bool,
}

impl App {
    fn new() -> Self {
        App {
            gui_app: GUIApp::new(),
            running: true,
        }
    }

    // 这些方法已移至GUIApp中实现

    fn update(&mut self, ctx: &egui::Context) {
        self.gui_app.update(ctx);
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ASCII Raycasting 3D Maze"),
        ..Default::default()
    };
    
    eframe::run_native(
        "ASCII Raycasting 3D Maze",
        options,
        Box::new(|_cc| Box::new(App::new())),
    ).expect("Failed to run application");
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update(ctx);
        
        // 处理退出
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.running = false;
        }
        
        if !self.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
