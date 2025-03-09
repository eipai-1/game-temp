use std::time::Duration;

/*
GameConfig
适用场景：存放游戏启动参数、核心逻辑配置（如世界大小、物理参数等）。
语义：强调“配置”，常用于初始化后不再频繁修改的静态参数。
---r1
 */
pub struct GameConfig {
    //最大帧数，为0时表示无限制
    //使用spin_sleeper库 精度应该满足要求
    max_fps: u32,
    pub sleeper: spin_sleep::SpinSleeper,
    frame_duration: Duration,
}

impl GameConfig {
    pub fn new() -> Self {
        let sleeper = spin_sleep::SpinSleeper::default();

        let max_fps = 0;
        let mut frame_duration = Duration::from_secs_f32(1.0);
        if max_fps != 0 {
            frame_duration = Duration::from_secs_f64(1.0 / max_fps as f64);
        }

        Self {
            max_fps,
            sleeper,
            frame_duration,
        }
    }

    #[allow(unused)]
    pub fn set_max_fps(&mut self, new_max_fps: u32) {
        self.max_fps = new_max_fps;
        if self.max_fps != 0 {
            self.frame_duration = Duration::from_secs_f32(1.0 / self.max_fps as f32);
        }
    }

    pub fn get_max_fps(&self) -> u32 {
        self.max_fps
    }

    pub fn get_frame_duration(&self) -> Duration {
        self.frame_duration
    }
}
