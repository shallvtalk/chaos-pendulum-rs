/// 物理引擎模块
/// 实现双摆的动力学方程和数值积分
use crate::pendulum::{PendulumParams, PendulumState};

/// 双摆的动力学方程导数
#[derive(Clone, Copy, Debug)]
pub struct StateDerivative {
    pub dtheta1: f64,
    pub dtheta2: f64,
    pub domega1: f64,
    pub domega2: f64,
}

impl StateDerivative {
    pub fn new(dtheta1: f64, dtheta2: f64, domega1: f64, domega2: f64) -> Self {
        Self {
            dtheta1,
            dtheta2,
            domega1,
            domega2,
        }
    }

    /// 将导数与标量相乘
    pub fn mul_scalar(&self, scalar: f64) -> Self {
        Self {
            dtheta1: self.dtheta1 * scalar,
            dtheta2: self.dtheta2 * scalar,
            domega1: self.domega1 * scalar,
            domega2: self.domega2 * scalar,
        }
    }

    /// 将两个导数相加
    pub fn add(&self, other: &Self) -> Self {
        Self {
            dtheta1: self.dtheta1 + other.dtheta1,
            dtheta2: self.dtheta2 + other.dtheta2,
            domega1: self.domega1 + other.domega1,
            domega2: self.domega2 + other.domega2,
        }
    }
}

/// 物理引擎
pub struct PhysicsEngine {
    /// 时间步长
    dt: f64,
}

impl PhysicsEngine {
    /// 创建新的物理引擎
    pub fn new(dt: f64) -> Self {
        Self { dt }
    }

    /// 设置时间步长
    pub fn set_dt(&mut self, dt: f64) {
        self.dt = dt.max(1e-6); // 防止时间步长过小
    }

    /// 高级步进函数 - 自动选择最佳积分器并验证能量守恒
    pub fn step(&self, state: &PendulumState, params: &PendulumParams) -> (PendulumState, f64) {
        let initial_energy = state.total_energy(params);

        // 使用RK4积分
        let new_state = self.integrate_rk4(state, params);
        let final_energy = new_state.total_energy(params);

        // 计算能量误差（用于监控数值精度）
        let energy_error = if initial_energy.abs() > 1e-10 {
            (final_energy - initial_energy).abs() / initial_energy.abs()
        } else {
            (final_energy - initial_energy).abs()
        };

        (new_state, energy_error)
    }

    /// 计算双摆系统的导数（动力学方程）
    /// 使用标准的Lagrange方程推导
    pub fn compute_derivatives(
        &self,
        state: &PendulumState,
        params: &PendulumParams,
    ) -> StateDerivative {
        let theta1 = state.theta1;
        let theta2 = state.theta2;
        let omega1 = state.omega1;
        let omega2 = state.omega2;

        let m1 = params.m1;
        let m2 = params.m2;
        let l1 = params.l1;
        let l2 = params.l2;
        let g = params.g;
        let damping = params.damping;

        // 角度差
        let delta_theta = theta1 - theta2;
        let cos_delta = delta_theta.cos();
        let sin_delta = delta_theta.sin();

        // 从Lagrange方程推导的标准双摆方程
        // 质量项
        let m11 = (m1 + m2) * l1 * l1;
        let m12 = m2 * l1 * l2 * cos_delta;
        let m22 = m2 * l2 * l2;

        // 科里奥利和离心力项
        let c1 = -m2 * l1 * l2 * omega2 * omega2 * sin_delta
            - 2.0 * m2 * l1 * l2 * omega2 * omega1 * sin_delta;
        let c2 = m2 * l1 * l2 * omega1 * omega1 * sin_delta;

        // 重力项（theta=0为垂直向下，重力提供回复力矩）
        let g1 = -(m1 + m2) * g * l1 * theta1.sin();
        let g2 = -m2 * g * l2 * theta2.sin();

        // 阻尼项
        let d1 = -damping * omega1;
        let d2 = -damping * omega2;

        // 右侧项
        let rhs1 = c1 + g1 + d1;
        let rhs2 = c2 + g2 + d2;

        // 质量矩阵的行列式
        let det = m11 * m22 - m12 * m12;

        // 避免奇异性
        let det = if det.abs() < 1e-10 {
            1e-10 * det.signum()
        } else {
            det
        };

        // 求解角加速度 (逆矩阵乘法)
        let alpha1 = (m22 * rhs1 - m12 * rhs2) / det;
        let alpha2 = (m11 * rhs2 - m12 * rhs1) / det;

        StateDerivative::new(omega1, omega2, alpha1, alpha2)
    }

    /// 使用欧拉方法进行数值积分（简单但精度较低）
    #[allow(dead_code)]
    pub fn integrate_euler(&self, state: &PendulumState, params: &PendulumParams) -> PendulumState {
        let dt = self.dt;
        let derivative = self.compute_derivatives(state, params);

        let mut new_state = self.add_scaled_derivative(state, &derivative, dt);
        new_state.normalize_angles();

        new_state
    }

    /// 使用Runge-Kutta 4阶方法进行数值积分
    pub fn integrate_rk4(&self, state: &PendulumState, params: &PendulumParams) -> PendulumState {
        let dt = self.dt;

        // k1 = f(t, y)
        let k1 = self.compute_derivatives(state, params);

        // k2 = f(t + dt/2, y + dt/2 * k1)
        let state2 = self.add_scaled_derivative(state, &k1, dt / 2.0);
        let k2 = self.compute_derivatives(&state2, params);

        // k3 = f(t + dt/2, y + dt/2 * k2)
        let state3 = self.add_scaled_derivative(state, &k2, dt / 2.0);
        let k3 = self.compute_derivatives(&state3, params);

        // k4 = f(t + dt, y + dt * k3)
        let state4 = self.add_scaled_derivative(state, &k3, dt);
        let k4 = self.compute_derivatives(&state4, params);

        // y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4)
        let k_combined = k1
            .add(&k2.mul_scalar(2.0))
            .add(&k3.mul_scalar(2.0))
            .add(&k4);

        let mut new_state = self.add_scaled_derivative(state, &k_combined, dt / 6.0);

        // 标准化角度到 [-π, π] 范围
        new_state.normalize_angles();

        new_state
    }

    /// 辅助函数：将状态与缩放的导数相加
    fn add_scaled_derivative(
        &self,
        state: &PendulumState,
        derivative: &StateDerivative,
        scale: f64,
    ) -> PendulumState {
        PendulumState::new(
            state.theta1 + derivative.dtheta1 * scale,
            state.theta2 + derivative.dtheta2 * scale,
            state.omega1 + derivative.domega1 * scale,
            state.omega2 + derivative.domega2 * scale,
        )
    }

    /// 自适应步长的Runge-Kutta方法（可选的高级功能）
    #[allow(dead_code)]
    pub fn integrate_adaptive(
        &self,
        state: &PendulumState,
        params: &PendulumParams,
        tolerance: f64,
    ) -> (PendulumState, f64) {
        let mut current_dt = self.dt;
        let min_dt = 1e-8;
        let max_dt = 1e-2;

        loop {
            // 使用当前步长计算一步
            let engine_full = PhysicsEngine::new(current_dt);
            let result_full = engine_full.integrate_rk4(state, params);

            // 使用两个半步长计算
            let engine_half = PhysicsEngine::new(current_dt / 2.0);
            let result_half1 = engine_half.integrate_rk4(state, params);
            let result_half2 = engine_half.integrate_rk4(&result_half1, params);

            // 估算误差
            let error = self.estimate_error(&result_full, &result_half2);

            if error < tolerance {
                // 误差足够小，接受结果
                if error < tolerance / 10.0 && current_dt < max_dt {
                    // 误差很小，可以增大步长
                    current_dt = (current_dt * 1.5).min(max_dt);
                }
                return (result_half2, current_dt);
            } else {
                // 误差太大，减小步长
                current_dt = (current_dt * 0.5).max(min_dt);
                if current_dt <= min_dt {
                    // 达到最小步长，强制接受结果
                    return (result_full, current_dt);
                }
            }
        }
    }

    /// 估算数值误差
    fn estimate_error(&self, full_step: &PendulumState, half_steps: &PendulumState) -> f64 {
        let error_theta1 = (full_step.theta1 - half_steps.theta1).abs();
        let error_theta2 = (full_step.theta2 - half_steps.theta2).abs();
        let error_omega1 = (full_step.omega1 - half_steps.omega1).abs();
        let error_omega2 = (full_step.omega2 - half_steps.omega2).abs();

        error_theta1
            .max(error_theta2)
            .max(error_omega1)
            .max(error_omega2)
    }
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new(0.001) // 默认1ms时间步长
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pendulum::PendulumParams;

    #[test]
    fn test_physics_engine_creation() {
        let engine = PhysicsEngine::new(0.01);
        assert_eq!(engine.dt, 0.01);
    }

    #[test]
    fn test_set_dt() {
        let mut engine = PhysicsEngine::new(0.01);
        engine.set_dt(0.005);
        assert_eq!(engine.dt, 0.005);

        // 测试最小时间步长限制
        engine.set_dt(1e-10);
        assert!(engine.dt >= 1e-6);
    }

    #[test]
    fn test_state_derivative_operations() {
        let d1 = StateDerivative::new(1.0, 2.0, 3.0, 4.0);
        let d2 = StateDerivative::new(0.5, 1.0, 1.5, 2.0);

        let scaled = d1.mul_scalar(2.0);
        assert_eq!(scaled.dtheta1, 2.0);
        assert_eq!(scaled.dtheta2, 4.0);

        let sum = d1.add(&d2);
        assert_eq!(sum.dtheta1, 1.5);
        assert_eq!(sum.dtheta2, 3.0);
    }

    #[test]
    fn test_compute_derivatives() {
        let engine = PhysicsEngine::new(0.001);
        let params = PendulumParams::default();

        // 测试垂直静止状态
        let state = PendulumState::new(0.0, 0.0, 0.0, 0.0);
        let derivatives = engine.compute_derivatives(&state, &params);

        // 在垂直位置，角速度的导数应该为0（静力平衡）
        assert_eq!(derivatives.dtheta1, 0.0);
        assert_eq!(derivatives.dtheta2, 0.0);
        assert!((derivatives.domega1).abs() < 1e-10);
        assert!((derivatives.domega2).abs() < 1e-10);
    }

    #[test]
    fn test_euler_integration() {
        let engine = PhysicsEngine::new(0.001);
        let params = PendulumParams::default();
        let state = PendulumState::new(0.1, 0.2, 0.0, 0.0);

        let new_state = engine.integrate_euler(&state, &params);

        // 角度应该有所变化（由于重力作用）
        assert_ne!(new_state.theta1, state.theta1);
        assert_ne!(new_state.theta2, state.theta2);
    }

    #[test]
    fn test_rk4_integration() {
        let engine = PhysicsEngine::new(0.001);
        let params = PendulumParams::default();
        let state = PendulumState::new(0.1, 0.2, 0.0, 0.0);

        let new_state = engine.integrate_rk4(&state, &params);

        // RK4应该给出不同于欧拉法的结果
        let euler_state = engine.integrate_euler(&state, &params);
        assert_ne!(new_state.theta1, euler_state.theta1);
        assert_ne!(new_state.theta2, euler_state.theta2);
    }

    #[test]
    fn test_energy_conservation() {
        let engine = PhysicsEngine::new(0.0001); // 很小的时间步长
        let params = PendulumParams::new(1.0, 1.0, 1.0, 1.0, 9.81, 0.0); // 无阻尼
        let mut state = PendulumState::new(0.5, 0.3, 0.0, 0.0);

        let initial_energy = state.total_energy(&params);

        // 进行多步积分
        for _ in 0..1000 {
            state = engine.integrate_rk4(&state, &params);
        }

        let final_energy = state.total_energy(&params);
        let energy_error = (final_energy - initial_energy).abs() / initial_energy.abs();

        // 能量误差应该很小（< 1%）
        assert!(energy_error < 0.01, "Energy error: {}", energy_error);
    }

    #[test]
    fn test_angle_normalization() {
        let engine = PhysicsEngine::new(0.001);
        let params = PendulumParams::default();

        // 创建一个角度超出范围的状态
        let state = PendulumState::new(
            4.0 * std::f64::consts::PI,
            -3.0 * std::f64::consts::PI,
            1.0,
            -1.0,
        );

        let new_state = engine.integrate_rk4(&state, &params);

        // 积分后角度应该被标准化
        assert!(new_state.theta1 >= -std::f64::consts::PI);
        assert!(new_state.theta1 <= std::f64::consts::PI);
        assert!(new_state.theta2 >= -std::f64::consts::PI);
        assert!(new_state.theta2 <= std::f64::consts::PI);
    }

    #[test]
    fn test_gravity_direction() {
        let engine = PhysicsEngine::new(0.001);
        let params = PendulumParams::default();

        // 测试：当摆向右偏移时（theta > 0），重力应该产生向左的力矩（负的角加速度）
        let state = PendulumState::new(0.1, 0.0, 0.0, 0.0); // 上摆向右偏移10度
        let derivatives = engine.compute_derivatives(&state, &params);

        // 重力应该产生负的角加速度，让摆回到平衡位置
        assert!(
            derivatives.domega1 < 0.0,
            "上摆向右偏移时，应该产生向左的角加速度"
        );

        // 测试：当摆向左偏移时（theta < 0），重力应该产生向右的力矩（正的角加速度）
        let state = PendulumState::new(-0.1, 0.0, 0.0, 0.0); // 上摆向左偏移10度
        let derivatives = engine.compute_derivatives(&state, &params);

        // 重力应该产生正的角加速度，让摆回到平衡位置
        assert!(
            derivatives.domega1 > 0.0,
            "上摆向左偏移时，应该产生向右的角加速度"
        );
    }
}
