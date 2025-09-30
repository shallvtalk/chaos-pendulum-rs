#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 导入模块
mod pendulum;
mod physics;
mod theme;
mod ui_state;
mod statistics;
mod visualization;
mod presets;

// 导入所需的外部crate
use eframe::egui;
use pendulum::{DoublePendulum, PendulumState, PendulumParams};
use physics::PhysicsEngine;
use statistics::PhysicsStatistics;
use theme::{ColorTheme, ThemeManager};
use ui_state::UiStateManager;
use visualization::PendulumRenderer;
use presets::get_all_presets;

/// 混沌双摆应用程序的主结构体
/// 包含物理系统、UI设置和控制参数
struct ChaosPendulumApp {
    /// 双摆物理系统
    pendulum: DoublePendulum,
    /// 物理引擎
    physics_engine: PhysicsEngine,
    /// 模拟是否正在运行
    is_running: bool,
    /// 上次更新的时间戳
    last_update: std::time::Instant,
    /// 更新间隔时间
    update_interval: std::time::Duration,
    /// 模拟速度倍率
    simulation_speed: f32,
    /// 时间步长设置
    time_step: f64,
    
    /// 物理统计管理器
    statistics: PhysicsStatistics,
    /// 主题管理器
    theme_manager: ThemeManager,
    /// UI状态管理器
    ui_state: UiStateManager,
    /// 可视化渲染器
    renderer: PendulumRenderer,
    
    /// 轨迹记录间隔（每N步记录一次）
    trajectory_record_interval: u32,
    /// 轨迹记录计数器
    trajectory_counter: u32,
    
    /// 参数调节的临时值
    temp_params: PendulumParams,
    /// 是否显示相空间图
    show_phase_space: bool,
    /// 是否显示能量图
    show_energy_plot: bool,
    /// 当前能量误差
    energy_error: f64,
}

impl Default for ChaosPendulumApp {
    fn default() -> Self {
        // 设置默认物理参数
        let params = PendulumParams::default();
        
        // 创建初始状态（向上偏移以获得足够势能）
        let initial_state = PendulumState::new(
            -std::f64::consts::PI / 6.0,  // 上摆向上30度
            -std::f64::consts::PI / 4.0,  // 下摆向上45度
            0.0,                          // 初始角速度为0
            0.0,
        );
        
        let pendulum = DoublePendulum::new(initial_state, params);
        let physics_engine = PhysicsEngine::new(0.001); // 1ms时间步长
        
        // 初始化统计数据
        let mut statistics = PhysicsStatistics::new(2000);
        let energy = pendulum.total_energy();
        statistics.add_energy_data(energy, pendulum.kinetic_energy(), pendulum.potential_energy());
        
        let (pos1, pos2) = pendulum.get_positions();
        statistics.add_trajectory_point(pos1.0, pos1.1, pos2.0, pos2.1);
        statistics.add_phase_space_point(
            pendulum.state.theta1, 
            pendulum.state.omega1, 
            pendulum.state.theta2, 
            pendulum.state.omega2
        );

        Self {
            pendulum,
            physics_engine,
            is_running: false,
            last_update: std::time::Instant::now(),
            update_interval: std::time::Duration::from_millis(16), // ~60 FPS
            simulation_speed: 1.0,
            time_step: 0.001,
            
            statistics,
            theme_manager: ThemeManager::new(ColorTheme::Dark),
            ui_state: UiStateManager::new(),
            renderer: PendulumRenderer::new(),
            
            trajectory_record_interval: 5, // 每5步记录一次轨迹点
            trajectory_counter: 0,
            
            temp_params: params,
            show_phase_space: false,
            show_energy_plot: true,
            energy_error: 0.0,
        }
    }
}

impl ChaosPendulumApp {
    /// 设置状态信息
    fn set_status(&mut self, message: String) {
        self.ui_state.set_status(message);
    }

    /// 更新状态信息
    fn update_status(&mut self) {
        self.ui_state.update_status();
    }

    /// 应用主题
    fn apply_theme(&self, ctx: &egui::Context) {
        self.theme_manager.apply_ui_theme(ctx);
    }

    /// 更新物理模拟
    fn update_physics(&mut self) {
        if !self.is_running {
            return;
        }

        let steps_per_frame = (self.simulation_speed * 10.0) as u32;
        
        for _ in 0..steps_per_frame {
            // 使用新的step函数更新物理状态并获取能量误差
            let (new_state, energy_err) = self.physics_engine.step(
                &self.pendulum.state,
                &self.pendulum.params,
            );
            self.pendulum.state = new_state;
            self.energy_error = energy_err;
            self.pendulum.advance_time(self.time_step);
            
            // 记录统计数据
            self.trajectory_counter += 1;
            if self.trajectory_counter >= self.trajectory_record_interval {
                self.trajectory_counter = 0;
                
                let energy = self.pendulum.total_energy();
                self.statistics.add_energy_data(
                    energy,
                    self.pendulum.kinetic_energy(),
                    self.pendulum.potential_energy(),
                );
                
                let (pos1, pos2) = self.pendulum.get_positions();
                self.statistics.add_trajectory_point(pos1.0, pos1.1, pos2.0, pos2.1);
                self.statistics.add_phase_space_point(
                    self.pendulum.state.theta1,
                    self.pendulum.state.omega1,
                    self.pendulum.state.theta2,
                    self.pendulum.state.omega2,
                );
            }
        }
    }

    /// 重置模拟
    fn reset_simulation(&mut self) {
        self.pendulum.reset(PendulumState::new(
            -std::f64::consts::PI / 6.0,
            -std::f64::consts::PI / 4.0,
            0.0,
            0.0,
        ));
        self.statistics.clear_history();
        self.trajectory_counter = 0;
        
        // 记录初始数据
        let energy = self.pendulum.total_energy();
        self.statistics.add_energy_data(energy, self.pendulum.kinetic_energy(), self.pendulum.potential_energy());
        
        let (pos1, pos2) = self.pendulum.get_positions();
        self.statistics.add_trajectory_point(pos1.0, pos1.1, pos2.0, pos2.1);
        self.statistics.add_phase_space_point(
            self.pendulum.state.theta1,
            self.pendulum.state.omega1,
            self.pendulum.state.theta2,
            self.pendulum.state.omega2,
        );
        
        self.set_status("Simulation reset".to_string());
    }

    /// 应用参数更改
    fn apply_parameters(&mut self) {
        match self.temp_params.validate() {
            Ok(_) => {
                self.pendulum.params = self.temp_params;
                self.set_status("Parameters updated".to_string());
            }
            Err(err) => {
                self.set_status(format!("Invalid parameters: {}", err));
            }
        }
    }

    /// 更新时间步长
    fn update_time_step(&mut self) {
        self.physics_engine.set_dt(self.time_step);
    }
}

impl eframe::App for ChaosPendulumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 更新主题动画
        self.theme_manager.update_theme_transition();
        
        // 应用主题
        self.apply_theme(ctx);
        
        // 更新状态信息
        self.update_status();
        
        // 处理键盘快捷键
        ctx.input(|i| {
            // Space - 开始/暂停
            if i.key_pressed(egui::Key::Space) {
                self.is_running = !self.is_running;
                self.last_update = std::time::Instant::now();
            }
            
            // R - 重置
            if i.key_pressed(egui::Key::R) {
                self.reset_simulation();
            }
            
            // T - 切换主题
            if i.key_pressed(egui::Key::T) {
                self.theme_manager.toggle_theme();
            }
        });

        // 检查是否需要更新物理模拟
        if self.is_running && self.last_update.elapsed() >= self.update_interval {
            self.update_physics();
            self.last_update = std::time::Instant::now();
            ctx.request_repaint(); // 请求重绘
        }

        // 创建左侧控制面板
        egui::SidePanel::left("controls").show(ctx, |ui| {
            ui.heading("🌀 Chaos Pendulum");
            ui.separator();
            
            // 模拟控制
            ui.collapsing("Simulation Control", |ui| {
                ui.horizontal(|ui| {
                    let play_text = if self.is_running { "⏸ Pause" } else { "▶ Play" };
                    if ui.button(play_text).clicked() {
                        self.is_running = !self.is_running;
                        self.last_update = std::time::Instant::now();
                    }
                    
                    if ui.button("🔄 Reset").clicked() {
                        self.reset_simulation();
                    }
                });
                
                ui.add(egui::Slider::new(&mut self.simulation_speed, 0.1..=5.0)
                    .text("Speed").logarithmic(false));
                
                ui.add(egui::Slider::new(&mut self.time_step, 0.0001..=0.01)
                    .text("Time Step").logarithmic(true));
                if ui.button("Apply Time Step").clicked() {
                    self.update_time_step();
                }
            });
            
            ui.separator();
            
            // 预设配置
            ui.collapsing("Presets", |ui| {
                let presets = get_all_presets();
                for preset in presets.iter() {
                    if ui.button(&preset.name).clicked() {
                        self.pendulum.state = preset.initial_state;
                        self.temp_params = preset.params;
                        self.pendulum.params = preset.params;
                        self.statistics.clear_history();
                        self.trajectory_counter = 0;
                        
                        // 记录初始数据
                        let energy = self.pendulum.total_energy();
                        self.statistics.add_energy_data(energy, self.pendulum.kinetic_energy(), self.pendulum.potential_energy());
                        
                        let (pos1, pos2) = self.pendulum.get_positions();
                        self.statistics.add_trajectory_point(pos1.0, pos1.1, pos2.0, pos2.1);
                        self.statistics.add_phase_space_point(
                            self.pendulum.state.theta1,
                            self.pendulum.state.omega1,
                            self.pendulum.state.theta2,
                            self.pendulum.state.omega2,
                        );
                        
                        self.set_status(format!("Loaded preset: {}", preset.name));
                    }
                    ui.small(&preset.description);
                }
            });
            
            ui.separator();
            
            // 物理参数
            ui.collapsing("Physical Parameters", |ui| {
                ui.add(egui::Slider::new(&mut self.temp_params.m1, 0.1..=5.0)
                    .text("Mass 1 (kg)"));
                ui.add(egui::Slider::new(&mut self.temp_params.m2, 0.1..=5.0)
                    .text("Mass 2 (kg)"));
                ui.add(egui::Slider::new(&mut self.temp_params.l1, 0.1..=3.0)
                    .text("Length 1 (m)"));
                ui.add(egui::Slider::new(&mut self.temp_params.l2, 0.1..=3.0)
                    .text("Length 2 (m)"));
                ui.add(egui::Slider::new(&mut self.temp_params.g, 1.0..=20.0)
                    .text("Gravity (m/s²)"));
                ui.add(egui::Slider::new(&mut self.temp_params.damping, 0.0..=1.0)
                    .text("Damping"));
                
                if ui.button("Apply Parameters").clicked() {
                    self.apply_parameters();
                }
            });
            
            ui.separator();
            
            // 显示选项
            ui.collapsing("Display Options", |ui| {
                let mut show_trajectory = self.ui_state.show_trajectory();
                ui.checkbox(&mut show_trajectory, "Show Trajectory");
                self.ui_state.set_show_trajectory(show_trajectory);
                
                ui.checkbox(&mut self.show_energy_plot, "Show Energy Plot");
                ui.checkbox(&mut self.show_phase_space, "Show Phase Space");
                
                let mut show_grid = self.ui_state.show_grid_lines();
                ui.checkbox(&mut show_grid, "Show Grid");
                self.ui_state.set_show_grid_lines(show_grid);
                
                if ui.button("🎨 Toggle Theme").clicked() {
                    self.theme_manager.toggle_theme();
                }
                
                ui.add(egui::Slider::new(&mut self.trajectory_record_interval, 1..=20)
                    .text("Trajectory Detail"));
                
                let mut alpha = self.ui_state.trajectory_alpha();
                ui.add(egui::Slider::new(&mut alpha, 0.1..=1.0)
                    .text("Trajectory Alpha"));
                self.ui_state.set_trajectory_alpha(alpha);
                
                if ui.button("Reset View").clicked() {
                    self.ui_state.reset_view();
                }
                
                let mut scale = self.renderer.scale();
                ui.add(egui::Slider::new(&mut scale, 20.0..=500.0)
                    .text("Zoom Scale").logarithmic(false));
                self.renderer.set_scale(scale);
                if ui.button("Reset Zoom").clicked() {
                    self.renderer.set_scale(100.0);
                }
            });
            
            ui.separator();
            
            // 状态信息
            if let Some(status) = self.ui_state.status_message() {
                ui.colored_label(egui::Color32::YELLOW, status);
            }
            
            // 实时信息显示
            ui.separator();
            ui.small(format!("Time: {:.2}s", self.pendulum.time));
            ui.small(format!("Total Energy: {:.3}J", self.pendulum.total_energy()));
            ui.small(format!("Kinetic: {:.3}J", self.pendulum.kinetic_energy()));
            ui.small(format!("Potential: {:.3}J", self.pendulum.potential_energy()));
            
            // 能量守恒监控
            ui.separator();
            let energy_color = if self.energy_error < 1e-6 {
                egui::Color32::GREEN
            } else if self.energy_error < 1e-4 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::RED
            };
            ui.colored_label(energy_color, format!("Energy Error: {:.2e}", self.energy_error));
        });

        // 创建右侧统计面板
        if self.show_energy_plot || self.show_phase_space {
            egui::SidePanel::right("statistics")
                .default_width(400.0)
                .min_width(300.0)
                .show(ctx, |ui| {
                ui.heading("📊 Analysis");
                
                if self.show_energy_plot && self.statistics.has_data() {
                    ui.collapsing("Energy Plot", |ui| {
                        use egui_plot::{Line, Plot, PlotPoints};
                        
                        let energy_history = self.statistics.get_energy_history();
                        if !energy_history.is_empty() {
                            let total_energy: PlotPoints = energy_history.iter()
                                .enumerate()
                                .map(|(i, (total, _, _))| [i as f64, *total])
                                .collect();
                            
                            let kinetic_energy: PlotPoints = energy_history.iter()
                                .enumerate()
                                .map(|(i, (_, kinetic, _))| [i as f64, *kinetic])
                                .collect();
                            
                            let potential_energy: PlotPoints = energy_history.iter()
                                .enumerate()
                                .map(|(i, (_, _, potential))| [i as f64, *potential])
                                .collect();
                            
                            Plot::new("energy_plot")
                                .height(250.0)
                                .show(ui, |plot_ui| {
                                    plot_ui.line(Line::new(total_energy).name("Total").color(egui::Color32::WHITE));
                                    plot_ui.line(Line::new(kinetic_energy).name("Kinetic").color(egui::Color32::RED));
                                    plot_ui.line(Line::new(potential_energy).name("Potential").color(egui::Color32::BLUE));
                                });
                        }
                    });
                }
                
                if self.show_phase_space && self.statistics.has_data() {
                    ui.collapsing("Phase Space", |ui| {
                        use egui_plot::{Line, Plot, PlotPoints};
                        
                        let phase_history = self.statistics.get_phase_space_history();
                        if !phase_history.is_empty() {
                            let phase_points1: PlotPoints = phase_history.iter()
                                .map(|(theta1, omega1, _, _)| [*theta1, *omega1])
                                .collect();
                            
                            let phase_points2: PlotPoints = phase_history.iter()
                                .map(|(_, _, theta2, omega2)| [*theta2, *omega2])
                                .collect();
                            
                            Plot::new("phase_space")
                                .height(250.0)
                                .show(ui, |plot_ui| {
                                    plot_ui.line(Line::new(phase_points1).name("Pendulum 1").color(egui::Color32::RED));
                                    plot_ui.line(Line::new(phase_points2).name("Pendulum 2").color(egui::Color32::BLUE));
                                });
                        }
                    });
                }
            });
        }

        // 创建中央面板用于显示摆的可视化
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🌀 Double Pendulum");
            
            // 显示当前状态信息
            ui.horizontal(|ui| {
                ui.label(format!("θ₁: {:.1}°", self.pendulum.state.theta1.to_degrees()));
                ui.separator();
                ui.label(format!("θ₂: {:.1}°", self.pendulum.state.theta2.to_degrees()));
                ui.separator();
                ui.label(format!("ω₁: {:.2}", self.pendulum.state.omega1));
                ui.separator();
                ui.label(format!("ω₂: {:.2}", self.pendulum.state.omega2));
            });
            
            ui.separator();
            
            // 渲染摆系统
            self.renderer.render(
                ui,
                &self.pendulum,
                &self.statistics,
                &self.theme_manager,
                &self.ui_state,
            );
        });

        // 如果模拟正在运行，请求持续重绘
        if self.is_running {
            ctx.request_repaint_after(self.update_interval);
        }

        // 如果正在进行主题切换动画，请求持续重绘
        if self.theme_manager.is_transitioning() {
            ctx.request_repaint();
        }
    }
}

/// 程序主入口函数
fn main() -> Result<(), eframe::Error> {
    // 配置应用程序窗口选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Chaos Double Pendulum Simulation"),
        ..Default::default()
    };

    // 启动应用程序
    eframe::run_native(
        "Chaos Double Pendulum",
        options,
        Box::new(|_cc| Ok(Box::new(ChaosPendulumApp::default()))),
    )
}