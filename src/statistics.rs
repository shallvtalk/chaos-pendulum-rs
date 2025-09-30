/// 物理统计模块
/// 负责跟踪和分析双摆的运动统计数据

#[allow(dead_code)]

/// 物理统计数据结构
#[derive(Clone, Debug)]
pub struct PhysicsStatistics {
    /// 能量历史记录（总能量、动能、势能）
    energy_history: Vec<(f64, f64, f64)>,
    /// 轨迹点历史记录 (x1, y1, x2, y2)
    trajectory_history: Vec<(f64, f64, f64, f64)>,
    /// 相空间点历史记录 (theta1, omega1, theta2, omega2)
    phase_space_history: Vec<(f64, f64, f64, f64)>,
    /// 历史记录的最大长度
    max_history_length: usize,
}

#[allow(dead_code)]
impl PhysicsStatistics {
    /// 创建新的物理统计实例
    pub fn new(max_history_length: usize) -> Self {
        Self {
            energy_history: Vec::new(),
            trajectory_history: Vec::new(),
            phase_space_history: Vec::new(),
            max_history_length,
        }
    }

    /// 添加新的能量数据点
    pub fn add_energy_data(
        &mut self,
        total_energy: f64,
        kinetic_energy: f64,
        potential_energy: f64,
    ) {
        self.energy_history
            .push((total_energy, kinetic_energy, potential_energy));

        // 保持历史记录在指定长度内
        if self.energy_history.len() > self.max_history_length {
            self.energy_history.remove(0);
        }
    }

    /// 添加新的轨迹数据点
    pub fn add_trajectory_point(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.trajectory_history.push((x1, y1, x2, y2));

        // 保持历史记录在指定长度内
        if self.trajectory_history.len() > self.max_history_length {
            self.trajectory_history.remove(0);
        }
    }

    /// 添加新的相空间数据点
    pub fn add_phase_space_point(&mut self, theta1: f64, omega1: f64, theta2: f64, omega2: f64) {
        self.phase_space_history
            .push((theta1, omega1, theta2, omega2));

        // 保持历史记录在指定长度内
        if self.phase_space_history.len() > self.max_history_length {
            self.phase_space_history.remove(0);
        }
    }

    /// 清除所有统计历史
    pub fn clear_history(&mut self) {
        self.energy_history.clear();
        self.trajectory_history.clear();
        self.phase_space_history.clear();
    }

    /// 获取能量历史记录的引用
    pub fn get_energy_history(&self) -> &Vec<(f64, f64, f64)> {
        &self.energy_history
    }

    /// 获取轨迹历史记录的引用
    pub fn get_trajectory_history(&self) -> &Vec<(f64, f64, f64, f64)> {
        &self.trajectory_history
    }

    /// 获取相空间历史记录的引用
    pub fn get_phase_space_history(&self) -> &Vec<(f64, f64, f64, f64)> {
        &self.phase_space_history
    }

    /// 获取当前历史记录长度
    pub fn get_history_length(&self) -> usize {
        self.energy_history.len()
    }

    /// 获取当前总能量
    pub fn get_current_total_energy(&self) -> Option<f64> {
        self.energy_history.last().map(|e| e.0)
    }

    /// 获取当前动能
    pub fn get_current_kinetic_energy(&self) -> Option<f64> {
        self.energy_history.last().map(|e| e.1)
    }

    /// 获取当前势能
    pub fn get_current_potential_energy(&self) -> Option<f64> {
        self.energy_history.last().map(|e| e.2)
    }

    /// 获取最大总能量
    pub fn get_max_total_energy(&self) -> Option<f64> {
        self.energy_history
            .iter()
            .map(|e| e.0)
            .fold(None, |acc, x| Some(acc.map_or(x, |y| x.max(y))))
    }

    /// 获取最小总能量
    pub fn get_min_total_energy(&self) -> Option<f64> {
        self.energy_history
            .iter()
            .map(|e| e.0)
            .fold(None, |acc, x| Some(acc.map_or(x, |y| x.min(y))))
    }

    /// 获取平均总能量
    pub fn get_average_total_energy(&self) -> Option<f64> {
        if self.energy_history.is_empty() {
            return None;
        }

        let sum: f64 = self.energy_history.iter().map(|e| e.0).sum();
        Some(sum / self.energy_history.len() as f64)
    }

    /// 检查是否有历史数据
    pub fn has_data(&self) -> bool {
        !self.energy_history.is_empty()
    }

    /// 计算能量守恒度（能量变化的标准差）
    pub fn get_energy_conservation(&self) -> Option<f64> {
        if self.energy_history.len() < 2 {
            return None;
        }

        let energies: Vec<f64> = self.energy_history.iter().map(|e| e.0).collect();
        let mean = energies.iter().sum::<f64>() / energies.len() as f64;
        let variance =
            energies.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / energies.len() as f64;

        Some(variance.sqrt())
    }

    /// 检测系统是否处于周期性运动
    /// 通过分析相空间轨迹的回归性来判断
    pub fn detect_periodicity(&self, tolerance: f64, min_period: usize) -> Option<usize> {
        if self.phase_space_history.len() < min_period * 2 {
            return None;
        }

        let history = &self.phase_space_history;
        let len = history.len();

        // 检查不同周期长度
        for period in min_period..len / 2 {
            let mut is_periodic = true;
            let check_points = (len / period).min(10); // 检查多个周期

            for i in 0..check_points {
                let idx1 = len - 1 - i * period;
                let idx2 = len - 1 - (i + 1) * period;

                if idx2 < period {
                    break;
                }

                let point1 = &history[idx1];
                let point2 = &history[idx2];

                // 检查四个维度的距离
                let distance = ((point1.0 - point2.0).powi(2)
                    + (point1.1 - point2.1).powi(2)
                    + (point1.2 - point2.2).powi(2)
                    + (point1.3 - point2.3).powi(2))
                .sqrt();

                if distance > tolerance {
                    is_periodic = false;
                    break;
                }
            }

            if is_periodic {
                return Some(period);
            }
        }

        None
    }

    /// 计算李雅普诺夫指数的近似值
    /// 通过观察相近初始条件的发散来估算
    pub fn estimate_lyapunov_exponent(&self, window_size: usize) -> Option<f64> {
        if self.phase_space_history.len() < window_size + 100 {
            return None;
        }

        let history = &self.phase_space_history;
        let len = history.len();
        let mut divergences = Vec::new();

        // 选择一个参考点和一个接近的点
        for i in 0..len - window_size {
            if i + window_size >= len {
                break;
            }

            let ref_point = &history[i];

            // 寻找最接近的点
            let mut min_distance = f64::INFINITY;
            let mut closest_idx = 0;

            for j in (i + 10)..(i + 50).min(len) {
                let test_point = &history[j];
                let distance = ((ref_point.0 - test_point.0).powi(2)
                    + (ref_point.1 - test_point.1).powi(2)
                    + (ref_point.2 - test_point.2).powi(2)
                    + (ref_point.3 - test_point.3).powi(2))
                .sqrt();

                if distance < min_distance && distance > 1e-8 {
                    min_distance = distance;
                    closest_idx = j;
                }
            }

            if closest_idx > 0 && closest_idx + window_size < len {
                let future_ref = &history[i + window_size];
                let future_close = &history[closest_idx + window_size];

                let future_distance = ((future_ref.0 - future_close.0).powi(2)
                    + (future_ref.1 - future_close.1).powi(2)
                    + (future_ref.2 - future_close.2).powi(2)
                    + (future_ref.3 - future_close.3).powi(2))
                .sqrt();

                if future_distance > 1e-8 && min_distance > 1e-8 {
                    let divergence_rate =
                        (future_distance / min_distance).ln() / window_size as f64;
                    divergences.push(divergence_rate);
                }
            }
        }

        if divergences.is_empty() {
            return None;
        }

        // 返回平均发散率
        Some(divergences.iter().sum::<f64>() / divergences.len() as f64)
    }
}

impl Default for PhysicsStatistics {
    fn default() -> Self {
        Self::new(2000) // 默认保存2000个数据点
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_statistics_creation() {
        let stats = PhysicsStatistics::new(100);
        assert_eq!(stats.max_history_length, 100);
        assert!(!stats.has_data());
    }

    #[test]
    fn test_add_energy_data() {
        let mut stats = PhysicsStatistics::new(3);
        stats.add_energy_data(100.0, 60.0, 40.0);
        stats.add_energy_data(102.0, 65.0, 37.0);
        stats.add_energy_data(99.0, 55.0, 44.0);

        assert_eq!(stats.get_history_length(), 3);
        assert!((stats.get_current_total_energy().unwrap() - 99.0).abs() < 1e-10);
        assert!((stats.get_max_total_energy().unwrap() - 102.0).abs() < 1e-10);
        assert!((stats.get_min_total_energy().unwrap() - 99.0).abs() < 1e-10);
    }

    #[test]
    fn test_history_length_limit() {
        let mut stats = PhysicsStatistics::new(2);
        stats.add_energy_data(100.0, 60.0, 40.0);
        stats.add_energy_data(102.0, 65.0, 37.0);
        stats.add_energy_data(99.0, 55.0, 44.0);

        assert_eq!(stats.get_history_length(), 2);
        assert_eq!(stats.get_energy_history().len(), 2);
    }

    #[test]
    fn test_clear_history() {
        let mut stats = PhysicsStatistics::new(10);
        stats.add_energy_data(100.0, 60.0, 40.0);
        stats.add_trajectory_point(1.0, 2.0, 3.0, 4.0);

        assert!(stats.has_data());
        stats.clear_history();
        assert!(!stats.has_data());
        assert!(stats.get_trajectory_history().is_empty());
    }

    #[test]
    fn test_energy_conservation() {
        let mut stats = PhysicsStatistics::new(10);
        // 添加能量守恒的数据
        for _ in 0..5 {
            stats.add_energy_data(100.0, 60.0, 40.0);
        }

        let conservation = stats.get_energy_conservation().unwrap();
        assert!(conservation < 1e-10); // 应该非常小

        // 添加能量不守恒的数据
        stats.add_energy_data(200.0, 120.0, 80.0);
        let conservation2 = stats.get_energy_conservation().unwrap();
        assert!(conservation2 > 10.0); // 应该较大
    }
}
