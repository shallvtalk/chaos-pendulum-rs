/// UI状态管理模块
/// 管理界面状态，包括显示选项和状态信息

#[derive(Clone, Debug)]
pub struct UiStateManager {
    /// 缩放级别
    zoom_level: f32,
    /// 视口平移偏移
    pan_offset: egui::Vec2,
    /// 是否显示网格线
    show_grid_lines: bool,
    /// 状态信息
    status_message: Option<String>,
    /// 状态信息显示的时间戳
    status_timestamp: Option<std::time::Instant>,
    /// 是否显示轨迹
    show_trajectory: bool,
    /// 轨迹透明度
    trajectory_alpha: f32,
}

impl UiStateManager {
    /// 创建新的UI状态管理器
    pub fn new() -> Self {
        Self {
            zoom_level: 1.0,
            pan_offset: egui::Vec2::ZERO,
            show_grid_lines: true,
            status_message: None,
            status_timestamp: None,
            show_trajectory: true,
            trajectory_alpha: 0.7,
        }
    }

    /// 重置视图设置
    pub fn reset_view(&mut self) {
        self.zoom_level = 1.0;
        self.pan_offset = egui::Vec2::ZERO;
    }

    /// 显示状态信息
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_timestamp = Some(std::time::Instant::now());
    }

    /// 更新状态信息（清除过期消息）
    pub fn update_status(&mut self) {
        if let Some(timestamp) = self.status_timestamp {
            if timestamp.elapsed().as_secs() > 3 {
                self.status_message = None;
                self.status_timestamp = None;
            }
        }
    }

    /// 获取当前状态信息
    pub fn status_message(&self) -> Option<&String> {
        self.status_message.as_ref()
    }

    /// 是否显示网格线
    pub fn show_grid_lines(&self) -> bool {
        self.show_grid_lines
    }

    /// 设置是否显示网格线
    pub fn set_show_grid_lines(&mut self, show: bool) {
        self.show_grid_lines = show;
    }

    /// 是否显示轨迹
    pub fn show_trajectory(&self) -> bool {
        self.show_trajectory
    }

    /// 设置是否显示轨迹
    pub fn set_show_trajectory(&mut self, show: bool) {
        self.show_trajectory = show;
    }

    /// 获取轨迹透明度
    pub fn trajectory_alpha(&self) -> f32 {
        self.trajectory_alpha
    }

    /// 设置轨迹透明度
    pub fn set_trajectory_alpha(&mut self, alpha: f32) {
        self.trajectory_alpha = alpha.clamp(0.0, 1.0);
    }
}

impl Default for UiStateManager {
    fn default() -> Self {
        Self::new()
    }
}