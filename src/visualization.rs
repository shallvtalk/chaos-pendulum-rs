/// 可视化渲染模块
/// 负责绘制双摆系统的实时状态和轨迹

#[allow(dead_code)]

use eframe::egui;
use crate::pendulum::DoublePendulum;
use crate::statistics::PhysicsStatistics;
use crate::theme::ThemeManager;
use crate::ui_state::UiStateManager;

/// 可视化渲染器
pub struct PendulumRenderer {
    /// 画布中心点
    center: egui::Pos2,
    /// 缩放比例（像素/米）
    scale: f32,
}

#[allow(dead_code)]
impl PendulumRenderer {
    /// 创建新的渲染器
    pub fn new() -> Self {
        Self {
            center: egui::Pos2::ZERO,
            scale: 100.0, // 默认100像素/米
        }
    }

    /// 在给定的UI区域内渲染摆系统
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        pendulum: &DoublePendulum,
        statistics: &PhysicsStatistics,
        theme_manager: &ThemeManager,
        ui_state: &UiStateManager,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        
        // 更新画布中心点
        self.center = available_rect.center();
        
        // 获取主题颜色
        let (rod_color, mass_color, trajectory_color, grid_color) = theme_manager.get_pendulum_colors();
        
        // 绘制背景网格
        if ui_state.show_grid_lines() {
            self.draw_grid(ui, available_rect, grid_color);
        }
        
        // 绘制轨迹历史
        if ui_state.show_trajectory() {
            self.draw_trajectory(ui, statistics, trajectory_color, ui_state.trajectory_alpha());
        }
        
        // 绘制悬挂点
        self.draw_suspension_point(ui, rod_color);
        
        // 绘制摆杆和质点
        self.draw_pendulum(ui, pendulum, rod_color, mass_color);
        
        // 处理鼠标交互
        self.handle_mouse_interaction(ui, ui_state);
    }

    /// 绘制背景网格
    fn draw_grid(&self, ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
        let painter = ui.painter();
        let grid_spacing = 50.0; // 网格间距（像素）
        
        // 垂直线
        let mut x = self.center.x - (self.center.x % grid_spacing);
        while x <= rect.max.x {
            if x >= rect.min.x {
                painter.line_segment(
                    [egui::Pos2::new(x, rect.min.y), egui::Pos2::new(x, rect.max.y)],
                    egui::Stroke::new(0.5, color),
                );
            }
            x += grid_spacing;
        }
        
        x = self.center.x;
        while x >= rect.min.x {
            painter.line_segment(
                [egui::Pos2::new(x, rect.min.y), egui::Pos2::new(x, rect.max.y)],
                egui::Stroke::new(0.5, color),
            );
            x -= grid_spacing;
        }
        
        // 水平线
        let mut y = self.center.y - (self.center.y % grid_spacing);
        while y <= rect.max.y {
            if y >= rect.min.y {
                painter.line_segment(
                    [egui::Pos2::new(rect.min.x, y), egui::Pos2::new(rect.max.x, y)],
                    egui::Stroke::new(0.5, color),
                );
            }
            y += grid_spacing;
        }
        
        y = self.center.y;
        while y >= rect.min.y {
            painter.line_segment(
                [egui::Pos2::new(rect.min.x, y), egui::Pos2::new(rect.max.x, y)],
                egui::Stroke::new(0.5, color),
            );
            y -= grid_spacing;
        }
    }

    /// 绘制轨迹历史
    fn draw_trajectory(&self, ui: &mut egui::Ui, statistics: &PhysicsStatistics, color: egui::Color32, alpha: f32) {
        let painter = ui.painter();
        let trajectory_history = statistics.get_trajectory_history();
        
        if trajectory_history.len() < 2 {
            return;
        }
        
        // 创建带透明度的颜色
        let trajectory_color = egui::Color32::from_rgba_premultiplied(
            color.r(),
            color.g(), 
            color.b(),
            (255.0 * alpha) as u8,
        );
        
        // 绘制第二个质点的轨迹
        let mut points = Vec::new();
        for (_, _, x2, y2) in trajectory_history {
            let screen_pos = self.world_to_screen(*x2, *y2);
            points.push(screen_pos);
        }
        
        // 绘制轨迹线段
        for i in 1..points.len() {
            let alpha_factor = i as f32 / points.len() as f32; // 渐变效果
            let line_color = egui::Color32::from_rgba_premultiplied(
                trajectory_color.r(),
                trajectory_color.g(),
                trajectory_color.b(),
                (trajectory_color.a() as f32 * alpha_factor) as u8,
            );
            
            painter.line_segment(
                [points[i-1], points[i]],
                egui::Stroke::new(1.5, line_color),
            );
        }
    }

    /// 绘制悬挂点
    fn draw_suspension_point(&self, ui: &mut egui::Ui, color: egui::Color32) {
        let painter = ui.painter();
        
        // 绘制悬挂点
        painter.circle_filled(self.center, 4.0, color);
        
        // 绘制悬挂支架
        let support_height = 20.0;
        painter.line_segment(
            [
                egui::Pos2::new(self.center.x - 15.0, self.center.y - support_height),
                egui::Pos2::new(self.center.x + 15.0, self.center.y - support_height),
            ],
            egui::Stroke::new(3.0, color),
        );
        painter.line_segment(
            [
                self.center - egui::Vec2::new(0.0, support_height),
                self.center,
            ],
            egui::Stroke::new(2.0, color),
        );
    }

    /// 绘制双摆系统
    fn draw_pendulum(&self, ui: &mut egui::Ui, pendulum: &DoublePendulum, rod_color: egui::Color32, mass_color: egui::Color32) {
        let painter = ui.painter();
        
        // 获取质点位置
        let (pos1, pos2) = pendulum.get_positions();
        let screen_pos1 = self.world_to_screen(pos1.0, pos1.1);
        let screen_pos2 = self.world_to_screen(pos2.0, pos2.1);
        
        // 绘制摆杆
        painter.line_segment(
            [self.center, screen_pos1],
            egui::Stroke::new(3.0, rod_color),
        );
        painter.line_segment(
            [screen_pos1, screen_pos2],
            egui::Stroke::new(3.0, rod_color),
        );
        
        // 计算质点大小（基于质量）
        let mass1_radius = (pendulum.params.m1 * 8.0 + 4.0) as f32;
        let mass2_radius = (pendulum.params.m2 * 8.0 + 4.0) as f32;
        
        // 绘制质点
        painter.circle_filled(screen_pos1, mass1_radius, mass_color);
        painter.circle_stroke(screen_pos1, mass1_radius, egui::Stroke::new(1.0, rod_color));
        
        painter.circle_filled(screen_pos2, mass2_radius, mass_color);
        painter.circle_stroke(screen_pos2, mass2_radius, egui::Stroke::new(1.0, rod_color));
        
        // 绘制速度向量（可选）
        self.draw_velocity_vectors(ui, pendulum, screen_pos1, screen_pos2, rod_color);
    }

    /// 绘制速度向量
    fn draw_velocity_vectors(&self, ui: &mut egui::Ui, pendulum: &DoublePendulum, pos1: egui::Pos2, pos2: egui::Pos2, color: egui::Color32) {
        let painter = ui.painter();
        
        // 计算线性速度
        let l1 = pendulum.params.l1;
        let l2 = pendulum.params.l2;
        let omega1 = pendulum.state.omega1;
        let omega2 = pendulum.state.omega2;
        let theta1 = pendulum.state.theta1;
        let theta2 = pendulum.state.theta2;
        
        // 上摆质点的速度
        let v1x = l1 * omega1 * theta1.cos();
        let v1y = l1 * omega1 * theta1.sin();
        
        // 下摆质点的速度
        let v2x = l1 * omega1 * theta1.cos() + l2 * omega2 * theta2.cos();
        let v2y = l1 * omega1 * theta1.sin() + l2 * omega2 * theta2.sin();
        
        // 缩放速度向量以便显示
        let velocity_scale = 10.0;
        
        // 创建半透明颜色
        let velocity_color = egui::Color32::from_rgba_premultiplied(
            color.r(),
            color.g(),
            color.b(),
            128,
        );
        
        // 绘制速度向量（注意Y轴翻转）
        if omega1.abs() > 0.01 {
            let v1_end = pos1 + egui::Vec2::new(v1x as f32 * velocity_scale, -v1y as f32 * velocity_scale);
            painter.arrow(pos1, v1_end - pos1, egui::Stroke::new(1.5, velocity_color));
        }
        
        if omega2.abs() > 0.01 {
            let v2_end = pos2 + egui::Vec2::new(v2x as f32 * velocity_scale, -v2y as f32 * velocity_scale);
            painter.arrow(pos2, v2_end - pos2, egui::Stroke::new(1.5, velocity_color));
        }
    }

    /// 处理鼠标交互（缩放和平移）
    fn handle_mouse_interaction(&mut self, ui: &mut egui::Ui, _ui_state: &UiStateManager) {
        let response = ui.interact(ui.available_rect_before_wrap(), ui.id(), egui::Sense::click_and_drag());
        
        // 处理滚轮缩放
        if let Some(_hover_pos) = response.hover_pos() {
            ui.input(|i| {
                let scroll_delta = i.smooth_scroll_delta.y;
                if scroll_delta != 0.0 {
                    let zoom_factor = 1.0 + scroll_delta * 0.001;
                    self.scale *= zoom_factor;
                    self.scale = self.scale.clamp(20.0, 500.0); // 限制缩放范围
                    
                    // 基于鼠标位置调整中心点 - 暂时忽略，只进行简单缩放
                }
            });
        }
        
        // 处理拖拽平移
        if response.dragged() {
            self.center += response.drag_delta();
        }
    }

    /// 世界坐标到屏幕坐标的转换
    fn world_to_screen(&self, world_x: f64, world_y: f64) -> egui::Pos2 {
        egui::Pos2::new(
            self.center.x + world_x as f32 * self.scale,
            self.center.y - world_y as f32 * self.scale, // 翻转Y轴：物理坐标Y向上，屏幕坐标Y向下
        )
    }

    /// 屏幕坐标到世界坐标的转换
    fn screen_to_world(&self, screen_pos: egui::Pos2) -> (f64, f64) {
        (
            ((screen_pos.x - self.center.x) / self.scale) as f64,
            ((self.center.y - screen_pos.y) / self.scale) as f64, // 翻转Y轴
        )
    }

    /// 重置视图
    pub fn reset_view(&mut self, ui_rect: egui::Rect) {
        self.center = ui_rect.center();
        self.scale = 100.0;
    }

    /// 获取当前缩放比例
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// 设置缩放比例
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.clamp(20.0, 500.0);
    }
}

impl Default for PendulumRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = PendulumRenderer::new();
        assert_eq!(renderer.scale, 100.0);
    }

    #[test]
    fn test_coordinate_transformation() {
        let renderer = PendulumRenderer::new();
        let world_pos = (1.0, -0.5);
        let screen_pos = renderer.world_to_screen(world_pos.0, world_pos.1);
        let back_to_world = renderer.screen_to_world(screen_pos);
        
        // 允许小的浮点误差
        assert!((back_to_world.0 - world_pos.0).abs() < 0.001);
        assert!((back_to_world.1 - world_pos.1).abs() < 0.001);
    }

    #[test]
    fn test_scale_limits() {
        let mut renderer = PendulumRenderer::new();
        
        renderer.set_scale(1000.0);
        assert_eq!(renderer.scale(), 500.0); // 应该被限制到最大值
        
        renderer.set_scale(5.0);
        assert_eq!(renderer.scale(), 20.0); // 应该被限制到最小值
    }
}