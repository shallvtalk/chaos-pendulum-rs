use crate::pendulum::DoublePendulum;
use crate::statistics::PhysicsStatistics;
use crate::theme::ThemeManager;
use crate::ui_state::UiStateManager;
/// 可视化渲染模块
/// 负责绘制双摆系统的实时状态和轨迹

#[allow(dead_code)]
use eframe::egui;

/// 可视化渲染器
pub struct PendulumRenderer {
    /// 画布中心点
    center: egui::Pos2,
    /// 缩放比例（像素/米）
    scale: f32,
    /// 当前正在拖动的摆球（None, Some(1), Some(2)）
    dragging_mass: Option<u8>,
    /// 拖动起始位置
    drag_start_pos: Option<egui::Pos2>,
}

#[allow(dead_code)]
impl PendulumRenderer {
    /// 创建新的渲染器
    pub fn new() -> Self {
        Self {
            center: egui::Pos2::ZERO,
            scale: 100.0, // 默认100像素/米
            dragging_mass: None,
            drag_start_pos: None,
        }
    }

    /// 在给定的UI区域内渲染摆系统
    /// 返回是否进行了拖动操作以及新的摆状态
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        pendulum: &DoublePendulum,
        statistics: &PhysicsStatistics,
        theme_manager: &ThemeManager,
        ui_state: &UiStateManager,
        is_paused: bool,
    ) -> Option<crate::pendulum::PendulumState> {
        let available_rect = ui.available_rect_before_wrap();

        // 更新画布中心点（只在第一次或重置时更新）
        if self.center == egui::Pos2::ZERO {
            self.center = available_rect.center();
        }

        // 先处理滚轮缩放
        self.handle_zoom(ui, available_rect);

        // 获取主题颜色
        let (rod_color, mass_color, trajectory_color, grid_color) =
            theme_manager.get_pendulum_colors();

        // 绘制背景网格
        if ui_state.show_grid_lines() {
            self.draw_grid(ui, available_rect, grid_color);
        }

        // 绘制轨迹历史
        if ui_state.show_trajectory() {
            self.draw_trajectory(
                ui,
                statistics,
                trajectory_color,
                ui_state.trajectory_alpha(),
            );
        }

        // 绘制悬挂点
        self.draw_suspension_point(ui, rod_color);

        // 绘制摆杆和质点
        self.draw_pendulum(ui, pendulum, rod_color, mass_color);

        // 处理鼠标交互（包括拖动）
        let new_state = if is_paused {
            // 在暂停状态下显示拖动提示
            self.draw_drag_hint(ui, pendulum);

            // 暂停时优先处理摆球拖动
            let pendulum_state = self.handle_pendulum_dragging(ui, pendulum);

            // 如果没有正在拖动摆球，则允许拖动画布
            if self.dragging_mass.is_none() {
                self.handle_canvas_pan(ui);
            }

            pendulum_state
        } else {
            // 运行时允许拖动画布平移
            self.handle_canvas_pan(ui);
            None
        };

        new_state
    }

    /// 绘制背景网格
    fn draw_grid(&self, ui: &mut egui::Ui, rect: egui::Rect, color: egui::Color32) {
        let painter = ui.painter();

        // 网格间距：物理空间中0.5米
        let grid_spacing_world = 0.5; // 0.5米

        // 计算网格起始位置（物理坐标）
        let world_min_x = ((rect.min.x - self.center.x) / self.scale) as f64;
        let world_max_x = ((rect.max.x - self.center.x) / self.scale) as f64;
        let world_min_y = ((self.center.y - rect.max.y) / self.scale) as f64;
        let world_max_y = ((self.center.y - rect.min.y) / self.scale) as f64;

        // 绘制垂直线
        let start_x = (world_min_x / grid_spacing_world).floor() * grid_spacing_world;
        let mut world_x = start_x;
        while world_x <= world_max_x {
            let screen_x = self.center.x + (world_x * self.scale as f64) as f32;
            if screen_x >= rect.min.x && screen_x <= rect.max.x {
                let stroke_width = if world_x.abs() < 0.01 { 1.0 } else { 0.5 };
                painter.line_segment(
                    [
                        egui::Pos2::new(screen_x, rect.min.y),
                        egui::Pos2::new(screen_x, rect.max.y),
                    ],
                    egui::Stroke::new(stroke_width, color),
                );
            }
            world_x += grid_spacing_world;
        }

        // 绘制水平线
        let start_y = (world_min_y / grid_spacing_world).floor() * grid_spacing_world;
        let mut world_y = start_y;
        while world_y <= world_max_y {
            let screen_y = self.center.y - (world_y * self.scale as f64) as f32;
            if screen_y >= rect.min.y && screen_y <= rect.max.y {
                let stroke_width = if world_y.abs() < 0.01 { 1.0 } else { 0.5 };
                painter.line_segment(
                    [
                        egui::Pos2::new(rect.min.x, screen_y),
                        egui::Pos2::new(rect.max.x, screen_y),
                    ],
                    egui::Stroke::new(stroke_width, color),
                );
            }
            world_y += grid_spacing_world;
        }
    }

    /// 绘制拖动提示
    fn draw_drag_hint(&self, ui: &mut egui::Ui, pendulum: &DoublePendulum) {
        let painter = ui.painter();

        // 获取摆球位置
        let (pos1, pos2) = pendulum.get_positions();
        let screen_pos1 = self.world_to_screen(pos1.0, pos1.1);
        let screen_pos2 = self.world_to_screen(pos2.0, pos2.1);

        // 计算摆球半径
        let mass1_radius = (pendulum.params.m1 * 8.0 + 4.0) as f32;
        let mass2_radius = (pendulum.params.m2 * 8.0 + 4.0) as f32;

        // 在摆球周围绘制虚线圆圈提示可以拖动
        let hint_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100);

        // 绘制提示圆圈
        painter.circle_stroke(
            screen_pos1,
            mass1_radius + 8.0,
            egui::Stroke::new(2.0, hint_color),
        );
        painter.circle_stroke(
            screen_pos2,
            mass2_radius + 8.0,
            egui::Stroke::new(2.0, hint_color),
        );

        // 显示文字提示
        if self.dragging_mass.is_none() {
            let hint_text = "Drag pendulum balls to adjust position";
            let text_pos = egui::Pos2::new(
                ui.available_rect_before_wrap().min.x + 10.0,
                ui.available_rect_before_wrap().min.y + 10.0,
            );
            painter.text(
                text_pos,
                egui::Align2::LEFT_TOP,
                hint_text,
                egui::FontId::default(),
                egui::Color32::LIGHT_GRAY,
            );
        }
    }

    /// 绘制轨迹历史
    fn draw_trajectory(
        &self,
        ui: &mut egui::Ui,
        statistics: &PhysicsStatistics,
        color: egui::Color32,
        alpha: f32,
    ) {
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
                [points[i - 1], points[i]],
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
    fn draw_pendulum(
        &self,
        ui: &mut egui::Ui,
        pendulum: &DoublePendulum,
        rod_color: egui::Color32,
        mass_color: egui::Color32,
    ) {
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

        // 绘制质点（拖动时使用不同颜色）
        let mass1_color = if self.dragging_mass == Some(1) {
            egui::Color32::YELLOW // 拖动时高亮显示
        } else {
            mass_color
        };
        let mass2_color = if self.dragging_mass == Some(2) {
            egui::Color32::YELLOW // 拖动时高亮显示
        } else {
            mass_color
        };

        painter.circle_filled(screen_pos1, mass1_radius, mass1_color);
        painter.circle_stroke(screen_pos1, mass1_radius, egui::Stroke::new(1.0, rod_color));

        painter.circle_filled(screen_pos2, mass2_radius, mass2_color);
        painter.circle_stroke(screen_pos2, mass2_radius, egui::Stroke::new(1.0, rod_color));

        // 绘制速度向量（可选）
        self.draw_velocity_vectors(ui, pendulum, screen_pos1, screen_pos2, rod_color);
    }

    /// 绘制速度向量
    fn draw_velocity_vectors(
        &self,
        ui: &mut egui::Ui,
        pendulum: &DoublePendulum,
        pos1: egui::Pos2,
        pos2: egui::Pos2,
        color: egui::Color32,
    ) {
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
        let velocity_color =
            egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 128);

        // 绘制速度向量（注意Y轴翻转）
        if omega1.abs() > 0.01 {
            let v1_end =
                pos1 + egui::Vec2::new(v1x as f32 * velocity_scale, -v1y as f32 * velocity_scale);
            painter.arrow(pos1, v1_end - pos1, egui::Stroke::new(1.5, velocity_color));
        }

        if omega2.abs() > 0.01 {
            let v2_end =
                pos2 + egui::Vec2::new(v2x as f32 * velocity_scale, -v2y as f32 * velocity_scale);
            painter.arrow(pos2, v2_end - pos2, egui::Stroke::new(1.5, velocity_color));
        }
    }

    /// 处理摆球拖动交互（仅在暂停状态下）
    fn handle_pendulum_dragging(
        &mut self,
        ui: &mut egui::Ui,
        pendulum: &crate::pendulum::DoublePendulum,
    ) -> Option<crate::pendulum::PendulumState> {
        // 获取当前摆球位置
        let (pos1, pos2) = pendulum.get_positions();
        let screen_pos1 = self.world_to_screen(pos1.0, pos1.1);
        let screen_pos2 = self.world_to_screen(pos2.0, pos2.1);

        // 计算摆球半径（用于检测点击）
        let mass1_radius = (pendulum.params.m1 * 8.0 + 4.0) as f32;
        let mass2_radius = (pendulum.params.m2 * 8.0 + 4.0) as f32;

        // 获取指针位置
        let pointer_pos = ui.ctx().pointer_interact_pos();

        // 检查是否正在拖动摆球
        if let Some(pos) = pointer_pos {
            // 开始拖动检测
            if ui.ctx().input(|i| i.pointer.primary_pressed()) && self.dragging_mass.is_none() {
                let dist1 = pos.distance(screen_pos1);
                let dist2 = pos.distance(screen_pos2);

                if dist1 <= mass1_radius + 5.0 {
                    self.dragging_mass = Some(1);
                    self.drag_start_pos = Some(pos);
                } else if dist2 <= mass2_radius + 5.0 {
                    self.dragging_mass = Some(2);
                    self.drag_start_pos = Some(pos);
                }
            }

            // 处理拖动过程
            if self.dragging_mass.is_some() && ui.ctx().input(|i| i.pointer.primary_down()) {
                let world_pos = self.screen_to_world(pos);
                return self.calculate_new_pendulum_state(pendulum, world_pos);
            }
        }

        // 拖动结束
        if ui.ctx().input(|i| i.pointer.primary_released()) {
            self.dragging_mass = None;
            self.drag_start_pos = None;
        }

        None
    }

    /// 根据拖动位置计算新的摆状态
    fn calculate_new_pendulum_state(
        &self,
        pendulum: &crate::pendulum::DoublePendulum,
        target_pos: (f64, f64),
    ) -> Option<crate::pendulum::PendulumState> {
        let l1 = pendulum.params.l1;
        let _l2 = pendulum.params.l2;

        match self.dragging_mass {
            Some(1) => {
                // 拖动上摆：计算新的theta1，保持theta2相对角度
                let new_theta1 = target_pos.0.atan2(-target_pos.1);
                let theta_diff = pendulum.state.theta2 - pendulum.state.theta1;
                let new_theta2 = new_theta1 + theta_diff;

                Some(crate::pendulum::PendulumState::new(
                    new_theta1, new_theta2, 0.0, // 拖动时重置角速度
                    0.0,
                ))
            }
            Some(2) => {
                // 拖动下摆：计算新的theta2，保持theta1不变
                let (pos1_x, pos1_y) = pendulum.state.get_mass1_position(l1);
                let relative_x = target_pos.0 - pos1_x;
                let relative_y = target_pos.1 - pos1_y;
                let new_theta2 = relative_x.atan2(-relative_y);

                Some(crate::pendulum::PendulumState::new(
                    pendulum.state.theta1,
                    new_theta2,
                    0.0, // 拖动时重置角速度
                    0.0,
                ))
            }
            _ => None, // 处理其他无效值
        }
    }

    /// 处理滚轮缩放
    fn handle_zoom(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let pointer_pos = ui.ctx().pointer_hover_pos();

        // 检查指针是否在可用区域内
        if let Some(pos) = pointer_pos {
            if rect.contains(pos) {
                ui.input(|i| {
                    // 使用 zoom_delta 或者计算 scroll_delta
                    let zoom = i.zoom_delta();
                    if (zoom - 1.0).abs() > 0.01 {
                        self.scale *= zoom;
                        self.scale = self.scale.clamp(20.0, 500.0);
                    }

                    // 备用方案：使用 scroll_delta
                    let scroll = i.raw_scroll_delta.y + i.smooth_scroll_delta.y;
                    if scroll.abs() > 0.1 {
                        let zoom_factor = 1.0 + scroll * 0.003;
                        self.scale *= zoom_factor;
                        self.scale = self.scale.clamp(20.0, 500.0);
                    }
                });
            }
        }
    }

    /// 处理画布平移（拖动）
    fn handle_canvas_pan(&mut self, ui: &mut egui::Ui) {
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("canvas_pan"),
            egui::Sense::click_and_drag(),
        );

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
    pub fn reset_view(&mut self) {
        self.center = egui::Pos2::ZERO; // 标记为需要重置
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
