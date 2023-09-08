use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // 预设图片路径
    // pub preset_image_path: String,
    // 预设配置路径
    // pub preset_config_path: String,
    // 预设名
    pub preset: Option<String>,
    // 总开关
    pub enable: bool,
    // 键盘按下点亮过渡时间(ms)
    pub key_down_transition_duration: u64,
    // 键盘抬起熄灭过渡时间(ms)
    pub key_up_transition_duration: u64,
    // 鼠标移动开关
    pub mouse_move_enable: bool,
    // 鼠标移动判断区间(从中心到边缘)
    pub mouse_move_radius_px: u64,
    // 鼠标移动时动画的过渡时间(ms)
    pub mouse_move_transition_duration: u64,
}

impl Config {
    pub fn set(&mut self, config: Config) -> () {
        self.preset = config.preset;
        self.enable = config.enable;
        self.key_down_transition_duration = config.key_down_transition_duration;
        self.key_up_transition_duration = config.key_up_transition_duration;
        self.mouse_move_enable = config.mouse_move_enable;
        self.mouse_move_radius_px = config.mouse_move_radius_px;
        self.mouse_move_transition_duration = config.mouse_move_transition_duration;
    }
}
