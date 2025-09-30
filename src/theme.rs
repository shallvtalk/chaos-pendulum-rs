/// 主题管理模块
/// 负责颜色主题切换和动画效果
use eframe::egui;

/// 颜色主题枚举
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ColorTheme {
    Light,
    Dark,
}

/// 主题管理器结构体
#[derive(Clone, Debug)]
pub struct ThemeManager {
    /// 当前颜色主题
    current_theme: ColorTheme,
    /// 主题切换动画进度 (0.0 到 1.0)
    transition_progress: f32,
    /// 主题切换开始时间
    transition_start: Option<std::time::Instant>,
    /// 目标主题（用于动画过渡）
    target_theme: ColorTheme,
}

impl ThemeManager {
    /// 创建新的主题管理器
    pub fn new(initial_theme: ColorTheme) -> Self {
        Self {
            current_theme: initial_theme,
            transition_progress: 1.0,
            transition_start: None,
            target_theme: initial_theme,
        }
    }


    /// 开始主题切换动画
    pub fn start_theme_transition(&mut self, new_theme: ColorTheme) {
        if new_theme != self.current_theme {
            self.target_theme = new_theme;
            self.transition_start = Some(std::time::Instant::now());
            self.transition_progress = 0.0;
        }
    }

    /// 更新主题切换动画
    pub fn update_theme_transition(&mut self) {
        if let Some(start_time) = self.transition_start {
            const TRANSITION_DURATION: f32 = 0.3; // 300ms 动画时长
            let elapsed = start_time.elapsed().as_secs_f32();
            self.transition_progress = (elapsed / TRANSITION_DURATION).min(1.0);

            // 使用缓动函数让动画更自然
            let eased_progress = Self::ease_in_out_cubic(self.transition_progress);
            self.transition_progress = eased_progress;

            // 动画完成后更新主题
            if self.transition_progress >= 1.0 {
                self.current_theme = self.target_theme;
                self.transition_start = None;
                self.transition_progress = 1.0;
            }
        }
    }

    /// 缓动函数：缓入缓出立方
    fn ease_in_out_cubic(t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }

    /// 检查是否正在进行主题切换动画
    pub fn is_transitioning(&self) -> bool {
        self.transition_progress < 1.0
    }

    /// 颜色插值函数
    fn lerp_color(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
        let t = t.clamp(0.0, 1.0);
        egui::Color32::from_rgb(
            (from.r() as f32 * (1.0 - t) + to.r() as f32 * t) as u8,
            (from.g() as f32 * (1.0 - t) + to.g() as f32 * t) as u8,
            (from.b() as f32 * (1.0 - t) + to.b() as f32 * t) as u8,
        )
    }

    /// 获取当前主题的颜色配置（用于摆杆和轨迹渲染）
    pub fn get_pendulum_colors(&self) -> (egui::Color32, egui::Color32, egui::Color32, egui::Color32) {
        let light_colors = (
            egui::Color32::BLACK,                  // 摆杆颜色
            egui::Color32::RED,                    // 质点颜色
            egui::Color32::BLUE,                   // 轨迹颜色
            egui::Color32::from_rgba_premultiplied(128, 128, 128, 128), // 背景网格
        );
        let dark_colors = (
            egui::Color32::WHITE,                  // 摆杆颜色
            egui::Color32::from_rgb(255, 100, 100), // 质点颜色
            egui::Color32::from_rgb(100, 150, 255), // 轨迹颜色
            egui::Color32::from_rgba_premultiplied(80, 80, 80, 128), // 背景网格
        );

        // 如果正在进行主题切换动画
        if self.transition_progress < 1.0 {
            let (from_colors, to_colors) = match self.target_theme {
                ColorTheme::Dark => (light_colors, dark_colors),
                ColorTheme::Light => (dark_colors, light_colors),
            };

            (
                Self::lerp_color(from_colors.0, to_colors.0, self.transition_progress),
                Self::lerp_color(from_colors.1, to_colors.1, self.transition_progress),
                Self::lerp_color(from_colors.2, to_colors.2, self.transition_progress),
                Self::lerp_color(from_colors.3, to_colors.3, self.transition_progress),
            )
        } else {
            match self.current_theme {
                ColorTheme::Light => light_colors,
                ColorTheme::Dark => dark_colors,
            }
        }
    }

    /// 设置UI主题（支持动画过渡）
    pub fn apply_ui_theme(&self, ctx: &egui::Context) {
        // 在动画过程中，根据进度选择UI主题
        let ui_theme = if self.transition_progress < 1.0 {
            if self.transition_progress < 0.5 {
                // 前半段使用当前主题
                self.current_theme
            } else {
                // 后半段使用目标主题
                self.target_theme
            }
        } else {
            self.current_theme
        };

        match ui_theme {
            ColorTheme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            ColorTheme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    /// 切换主题
    pub fn toggle_theme(&mut self) {
        let new_theme = match self.current_theme {
            ColorTheme::Light => ColorTheme::Dark,
            ColorTheme::Dark => ColorTheme::Light,
        };
        self.start_theme_transition(new_theme);
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new(ColorTheme::Dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let theme_manager = ThemeManager::new(ColorTheme::Light);
        assert_eq!(theme_manager.current_theme(), ColorTheme::Light);
        assert!(!theme_manager.is_transitioning());
    }

    #[test]
    fn test_theme_toggle() {
        let mut theme_manager = ThemeManager::new(ColorTheme::Light);
        theme_manager.toggle_theme();

        // 应该开始过渡到Dark主题
        assert!(theme_manager.is_transitioning());
        assert_eq!(theme_manager.target_theme, ColorTheme::Dark);
    }

    #[test]
    fn test_color_interpolation() {
        let color1 = egui::Color32::from_rgb(0, 0, 0);
        let color2 = egui::Color32::from_rgb(255, 255, 255);

        let mid_color = ThemeManager::lerp_color(color1, color2, 0.5);
        assert_eq!(mid_color, egui::Color32::from_rgb(127, 127, 127));
    }
}