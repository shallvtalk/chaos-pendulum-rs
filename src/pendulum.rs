/// 双摆物理系统模块
/// 定义双摆的状态、参数和基本物理计算
use serde::{Deserialize, Serialize};

/// 双摆的瞬时状态
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendulumState {
    /// 上摆的角度（弧度）
    pub theta1: f64,
    /// 下摆的角度（弧度）
    pub theta2: f64,
    /// 上摆的角速度（弧度/秒）
    pub omega1: f64,
    /// 下摆的角速度（弧度/秒）
    pub omega2: f64,
}

impl PendulumState {
    /// 创建新的摆状态
    pub fn new(theta1: f64, theta2: f64, omega1: f64, omega2: f64) -> Self {
        Self {
            theta1,
            theta2,
            omega1,
            omega2,
        }
    }

    /// 获取上摆质点的笛卡尔坐标
    pub fn get_mass1_position(&self, l1: f64) -> (f64, f64) {
        let x1 = l1 * self.theta1.sin();
        let y1 = -l1 * self.theta1.cos();
        (x1, y1)
    }

    /// 获取下摆质点的笛卡尔坐标
    pub fn get_mass2_position(&self, l1: f64, l2: f64) -> (f64, f64) {
        let (x1, y1) = self.get_mass1_position(l1);
        let x2 = x1 + l2 * self.theta2.sin();
        let y2 = y1 - l2 * self.theta2.cos();
        (x2, y2)
    }

    /// 计算系统的动能
    pub fn kinetic_energy(&self, params: &PendulumParams) -> f64 {
        let m1 = params.m1;
        let m2 = params.m2;
        let l1 = params.l1;
        let l2 = params.l2;
        let omega1 = self.omega1;
        let omega2 = self.omega2;
        let theta1 = self.theta1;
        let theta2 = self.theta2;

        // 上摆动能
        let ke1 = 0.5 * m1 * l1.powi(2) * omega1.powi(2);

        // 下摆动能（包括平移和旋转）
        let v2x = l1 * omega1 * theta1.cos() + l2 * omega2 * theta2.cos();
        let v2y = l1 * omega1 * theta1.sin() + l2 * omega2 * theta2.sin();
        let ke2 = 0.5 * m2 * (v2x.powi(2) + v2y.powi(2));

        ke1 + ke2
    }

    /// 计算系统的势能
    pub fn potential_energy(&self, params: &PendulumParams) -> f64 {
        let m1 = params.m1;
        let m2 = params.m2;
        let l1 = params.l1;
        let l2 = params.l2;
        let g = params.g;

        // 势能参考点为摆的悬挂点
        let y1 = -l1 * self.theta1.cos();
        let y2 = y1 - l2 * self.theta2.cos();

        m1 * g * y1 + m2 * g * y2
    }

    /// 计算系统的总能量
    pub fn total_energy(&self, params: &PendulumParams) -> f64 {
        self.kinetic_energy(params) + self.potential_energy(params)
    }

    /// 标准化角度到 [-π, π] 范围
    pub fn normalize_angles(&mut self) {
        self.theta1 = normalize_angle(self.theta1);
        self.theta2 = normalize_angle(self.theta2);
    }

    /// 创建静止状态（角速度为0）
    #[allow(dead_code)]
    pub fn at_rest(theta1: f64, theta2: f64) -> Self {
        Self::new(theta1, theta2, 0.0, 0.0)
    }
}

/// 双摆的物理参数
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PendulumParams {
    /// 上摆质量（kg）
    pub m1: f64,
    /// 下摆质量（kg）
    pub m2: f64,
    /// 上摆长度（m）
    pub l1: f64,
    /// 下摆长度（m）
    pub l2: f64,
    /// 重力加速度（m/s²）
    pub g: f64,
    /// 阻尼系数
    pub damping: f64,
}

impl PendulumParams {
    /// 创建新的摆参数
    pub fn new(m1: f64, m2: f64, l1: f64, l2: f64, g: f64, damping: f64) -> Self {
        Self {
            m1,
            m2,
            l1,
            l2,
            g,
            damping,
        }
    }

    /// 验证参数是否有效
    pub fn validate(&self) -> Result<(), String> {
        if self.m1 <= 0.0 {
            return Err("上摆质量必须为正数".to_string());
        }
        if self.m2 <= 0.0 {
            return Err("下摆质量必须为正数".to_string());
        }
        if self.l1 <= 0.0 {
            return Err("上摆长度必须为正数".to_string());
        }
        if self.l2 <= 0.0 {
            return Err("下摆长度必须为正数".to_string());
        }
        if self.g <= 0.0 {
            return Err("重力加速度必须为正数".to_string());
        }
        if self.damping < 0.0 {
            return Err("阻尼系数不能为负数".to_string());
        }
        Ok(())
    }
}

impl Default for PendulumParams {
    fn default() -> Self {
        Self::new(
            1.0,  // m1: 1kg
            1.0,  // m2: 1kg
            1.0,  // l1: 1m
            1.0,  // l2: 1m
            9.81, // g: 9.81 m/s²
            0.0,  // damping: 无阻尼
        )
    }
}

/// 标准化角度到 [-π, π] 范围
pub fn normalize_angle(angle: f64) -> f64 {
    let mut normalized = angle % (2.0 * std::f64::consts::PI);
    if normalized > std::f64::consts::PI {
        normalized -= 2.0 * std::f64::consts::PI;
    } else if normalized < -std::f64::consts::PI {
        normalized += 2.0 * std::f64::consts::PI;
    }
    normalized
}

/// 双摆系统的完整状态和参数组合
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoublePendulum {
    /// 当前状态
    pub state: PendulumState,
    /// 物理参数
    pub params: PendulumParams,
    /// 模拟时间
    pub time: f64,
}

impl DoublePendulum {
    /// 创建新的双摆系统
    pub fn new(state: PendulumState, params: PendulumParams) -> Self {
        Self {
            state,
            params,
            time: 0.0,
        }
    }

    /// 重置到新状态
    pub fn reset(&mut self, new_state: PendulumState) {
        self.state = new_state;
        self.time = 0.0;
    }

    /// 更新模拟时间
    pub fn advance_time(&mut self, dt: f64) {
        self.time += dt;
    }

    /// 获取当前总能量
    pub fn total_energy(&self) -> f64 {
        self.state.total_energy(&self.params)
    }

    /// 获取当前动能
    pub fn kinetic_energy(&self) -> f64 {
        self.state.kinetic_energy(&self.params)
    }

    /// 获取当前势能
    pub fn potential_energy(&self) -> f64 {
        self.state.potential_energy(&self.params)
    }

    /// 获取两个质点的当前位置
    pub fn get_positions(&self) -> ((f64, f64), (f64, f64)) {
        let pos1 = self.state.get_mass1_position(self.params.l1);
        let pos2 = self
            .state
            .get_mass2_position(self.params.l1, self.params.l2);
        (pos1, pos2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pendulum_state_creation() {
        let state = PendulumState::new(1.0, 2.0, 0.5, -0.5);
        assert_eq!(state.theta1, 1.0);
        assert_eq!(state.theta2, 2.0);
        assert_eq!(state.omega1, 0.5);
        assert_eq!(state.omega2, -0.5);
    }

    #[test]
    fn test_at_rest_state() {
        let state = PendulumState::at_rest(std::f64::consts::PI / 4.0, std::f64::consts::PI / 6.0);
        assert_eq!(state.omega1, 0.0);
        assert_eq!(state.omega2, 0.0);
    }

    #[test]
    fn test_position_calculation() {
        let state = PendulumState::new(0.0, 0.0, 0.0, 0.0); // 垂直向下
        let (x1, y1) = state.get_mass1_position(1.0);
        let (x2, y2) = state.get_mass2_position(1.0, 1.0);

        assert!((x1 - 0.0).abs() < 1e-10);
        assert!((y1 - (-1.0)).abs() < 1e-10);
        assert!((x2 - 0.0).abs() < 1e-10);
        assert!((y2 - (-2.0)).abs() < 1e-10);
    }

    #[test]
    fn test_energy_conservation() {
        let params = PendulumParams::default();
        let state = PendulumState::at_rest(std::f64::consts::PI / 4.0, std::f64::consts::PI / 4.0);

        let ke = state.kinetic_energy(&params);
        let pe = state.potential_energy(&params);
        let total = state.total_energy(&params);

        assert!((ke - 0.0).abs() < 1e-10); // 静止状态动能为0
        assert!(pe < 0.0); // 势能应该为负（低于参考点）
        assert!((total - (ke + pe)).abs() < 1e-10); // 总能量 = 动能 + 势能
    }

    #[test]
    fn test_parameter_validation() {
        let valid_params = PendulumParams::default();
        assert!(valid_params.validate().is_ok());

        let invalid_params = PendulumParams::new(-1.0, 1.0, 1.0, 1.0, 9.81, 0.0);
        assert!(invalid_params.validate().is_err());
    }

    #[test]
    fn test_angle_normalization() {
        assert!((normalize_angle(0.0) - 0.0).abs() < 1e-10);
        assert!((normalize_angle(std::f64::consts::PI) - std::f64::consts::PI).abs() < 1e-10);
        assert!((normalize_angle(-std::f64::consts::PI) - (-std::f64::consts::PI)).abs() < 1e-10);

        // 测试大角度
        let big_angle = 3.0 * std::f64::consts::PI;
        let normalized = normalize_angle(big_angle);
        assert!(normalized > -std::f64::consts::PI && normalized <= std::f64::consts::PI);
    }

    #[test]
    fn test_double_pendulum_system() {
        let state = PendulumState::at_rest(0.1, 0.2);
        let params = PendulumParams::default();
        let mut pendulum = DoublePendulum::new(state, params);

        assert_eq!(pendulum.time, 0.0);

        pendulum.advance_time(0.1);
        assert_eq!(pendulum.time, 0.1);

        let energy = pendulum.total_energy();
        assert!(energy < 0.0); // 由于位于参考点下方
    }
}
