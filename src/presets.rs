/// 预设配置模块
/// 提供经典的混沌摆初始条件和参数组合

#[allow(dead_code)]

use crate::pendulum::{PendulumState, PendulumParams};
use serde::{Deserialize, Serialize};

/// 预设配置结构体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendulumPreset {
    /// 预设名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 初始状态
    pub initial_state: PendulumState,
    /// 物理参数
    pub params: PendulumParams,
}

impl PendulumPreset {
    /// 创建新的预设
    pub fn new(
        name: String,
        description: String,
        initial_state: PendulumState,
        params: PendulumParams,
    ) -> Self {
        Self {
            name,
            description,
            initial_state,
            params,
        }
    }
}

/// 获取所有预设配置
pub fn get_all_presets() -> Vec<PendulumPreset> {
    vec![
        // 小角度摆动
        PendulumPreset::new(
            "Small Angle".to_string(),
            "Small angle oscillation - predictable behavior".to_string(),
            PendulumState::new(-0.2, -0.1, 0.0, 0.0), // 向上偏移，有足够势能
            PendulumParams::default(),
        ),
        
        // 经典混沌
        PendulumPreset::new(
            "Classic Chaos".to_string(),
            "Classic chaotic motion with equal masses".to_string(),
            PendulumState::new(
                -std::f64::consts::PI / 2.0, // 向上90度
                -std::f64::consts::PI / 3.0, // 向上60度
                0.0,
                0.0,
            ),
            PendulumParams::default(),
        ),
        
        // 高能量混沌
        PendulumPreset::new(
            "High Energy".to_string(),
            "High energy chaotic motion - complex trajectories".to_string(),
            PendulumState::new(
                -std::f64::consts::PI * 0.7, // 向上126度
                -std::f64::consts::PI * 0.8, // 向上144度
                1.0,   // 减少初始角速度避免过快
                -0.5,
            ),
            PendulumParams::default(),
        ),
        
        // 不等质量
        PendulumPreset::new(
            "Unequal Masses".to_string(),
            "Heavy bottom mass creates interesting dynamics".to_string(),
            PendulumState::new(
                -std::f64::consts::PI / 3.0, // 向上60度
                -std::f64::consts::PI / 4.0, // 向上45度
                0.0,
                0.0,
            ),
            PendulumParams::new(1.0, 3.0, 1.0, 1.0, 9.81, 0.0),
        ),
        
        // 不等长度
        PendulumPreset::new(
            "Unequal Lengths".to_string(),
            "Different arm lengths create asymmetric motion".to_string(),
            PendulumState::new(
                -std::f64::consts::PI / 4.0, // 向上45度
                -std::f64::consts::PI / 3.0, // 向上60度
                0.0,
                0.0,
            ),
            PendulumParams::new(1.0, 1.0, 1.5, 0.8, 9.81, 0.0),
        ),
        
        // 阻尼系统
        PendulumPreset::new(
            "Damped System".to_string(),
            "Damped motion shows energy dissipation".to_string(),
            PendulumState::new(
                -std::f64::consts::PI / 2.0, // 向上90度
                -std::f64::consts::PI / 4.0, // 向上45度
                0.0,
                0.0,
            ),
            PendulumParams::new(1.0, 1.0, 1.0, 1.0, 9.81, 0.1),
        ),
        
        // 低重力
        PendulumPreset::new(
            "Low Gravity".to_string(),
            "Moon-like gravity creates slower, extended motion".to_string(),
            PendulumState::new(
                -std::f64::consts::PI / 3.0, // 向上60度
                -std::f64::consts::PI / 2.0, // 向上90度
                0.0,
                0.0,
            ),
            PendulumParams::new(1.0, 1.0, 1.0, 1.0, 1.62, 0.0), // 月球重力
        ),
        
        // 近似圆周运动
        PendulumPreset::new(
            "Near Circular".to_string(),
            "High initial velocity creates near-circular motion".to_string(),
            PendulumState::new(0.0, 0.0, 3.0, 4.0),
            PendulumParams::default(),
        ),
    ]
}

/// 根据名称获取预设
#[allow(dead_code)]
pub fn get_preset_by_name(name: &str) -> Option<PendulumPreset> {
    get_all_presets().into_iter().find(|preset| preset.name == name)
}

/// 预设类别
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum PresetCategory {
    Basic,
    Chaotic,
    Physical,
    Extreme,
}

/// 获取分类的预设
#[allow(dead_code)]
pub fn get_presets_by_category(category: PresetCategory) -> Vec<PendulumPreset> {
    let all_presets = get_all_presets();
    
    match category {
        PresetCategory::Basic => vec![
            all_presets[0].clone(), // Small Angle
            all_presets[1].clone(), // Classic Chaos
        ],
        PresetCategory::Chaotic => vec![
            all_presets[1].clone(), // Classic Chaos
            all_presets[2].clone(), // High Energy
            all_presets[7].clone(), // Near Circular
        ],
        PresetCategory::Physical => vec![
            all_presets[3].clone(), // Unequal Masses
            all_presets[4].clone(), // Unequal Lengths
            all_presets[5].clone(), // Damped System
        ],
        PresetCategory::Extreme => vec![
            all_presets[2].clone(), // High Energy
            all_presets[6].clone(), // Low Gravity
            all_presets[7].clone(), // Near Circular
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_presets() {
        let presets = get_all_presets();
        assert!(!presets.is_empty());
        assert!(presets.len() >= 8);
    }

    #[test]
    fn test_get_preset_by_name() {
        let preset = get_preset_by_name("Small Angle");
        assert!(preset.is_some());
        assert_eq!(preset.unwrap().name, "Small Angle");
        
        let none_preset = get_preset_by_name("Nonexistent");
        assert!(none_preset.is_none());
    }

    #[test]
    fn test_preset_validation() {
        let presets = get_all_presets();
        for preset in presets {
            assert!(preset.params.validate().is_ok());
            assert!(!preset.name.is_empty());
            assert!(!preset.description.is_empty());
        }
    }

    #[test]
    fn test_categories() {
        let basic = get_presets_by_category(PresetCategory::Basic);
        let chaotic = get_presets_by_category(PresetCategory::Chaotic);
        let physical = get_presets_by_category(PresetCategory::Physical);
        let extreme = get_presets_by_category(PresetCategory::Extreme);
        
        assert!(!basic.is_empty());
        assert!(!chaotic.is_empty());
        assert!(!physical.is_empty());
        assert!(!extreme.is_empty());
    }
}