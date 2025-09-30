# 🌀 Chaos Double Pendulum Simulation

一个用Rust和egui构建的双摆混沌系统模拟器，展示复杂的非线性动力学行为。

## ✨ 特性

- **实时物理模拟**: 使用Runge-Kutta 4阶积分器进行高精度数值计算
- **可视化系统**: 实时渲染双摆运动轨迹和能量变化
- **参数控制**: 可调节质量、长度、重力和阻尼参数
- **预设配置**: 内置多种经典混沌摆初始条件
- **统计分析**: 能量守恒监控、相空间图和轨迹分析
- **主题系统**: 支持明暗主题切换

## 🚀 快速开始

### 依赖要求

- Rust 1.70+
- Windows/Linux/macOS

### 安装运行

```bash
# 克隆仓库
git clone https://github.com/shallvtalk/chaos-pendulum-rs.git
cd chaos-pendulum-rs

# 编译运行
cargo run --release
```

## 🎮 使用方法

### 控制按键
- `Space` - 开始/暂停模拟
- `R` - 重置摆到初始状态
- `T` - 切换明暗主题

### 参数调节
- **质量**: 调节上下摆的质量(kg)
- **长度**: 调节摆杆长度(m) 
- **重力**: 调节重力加速度(m/s²)
- **阻尼**: 调节阻尼系数
- **时间步长**: 调节数值积分精度

### 预设配置
- **Small Angle** - 小角度振荡，行为可预测
- **Classic Chaos** - 经典混沌运动
- **High Energy** - 高能量复杂轨迹
- **Unequal Masses** - 不等质量动力学
- **Unequal Lengths** - 不等长度非对称运动
- **Damped System** - 阻尼能量耗散
- **Low Gravity** - 月球重力环境
- **Near Circular** - 近似圆周运动

## 🔬 物理原理

双摆系统是研究混沌理论的经典模型，其动力学方程基于拉格朗日力学：

- 使用广义坐标θ₁、θ₂描述两个摆的角度
- 通过拉格朗日方程L = T - V推导运动方程
- 实现Runge-Kutta 4阶数值积分求解
- 监控总能量守恒验证数值精度

## 🏗️ 项目结构

```
src/
├── main.rs           # 主应用入口
├── pendulum.rs       # 双摆物理模型
├── physics.rs        # 物理引擎和数值积分
├── visualization.rs  # 可视化渲染
├── statistics.rs     # 统计分析
├── theme.rs          # 主题管理
├── ui_state.rs       # UI状态管理
└── presets.rs        # 预设配置
```

## 🎯 技术特点

- **高性能**: Rust零成本抽象，优化的数值计算
- **实时渲染**: egui immediate mode GUI，流畅的60fps显示
- **精确物理**: RK4积分器保证数值稳定性和能量守恒
- **模块化设计**: 清晰的代码架构，易于扩展
- **跨平台**: 支持Windows、Linux、macOS

## 📊 可视化功能

- **实时摆动画**: 动态显示双摆运动状态
- **轨迹绘制**: 记录和显示质点运动轨迹
- **能量图表**: 监控动能、势能和总能量变化
- **相空间图**: 显示系统在相空间的演化
- **参数面板**: 实时调节物理参数

## 🤝 贡献

欢迎提交Issue和Pull Request！

## 📄 许可证

MIT License

## 🔗 相关资源

- [混沌理论入门](https://en.wikipedia.org/wiki/Chaos_theory)
- [双摆动力学](https://en.wikipedia.org/wiki/Double_pendulum)
- [Runge-Kutta方法](https://en.wikipedia.org/wiki/Runge%E2%80%93Kutta_methods)

---

🌀 *探索混沌之美，感受非线性动力学的魅力*