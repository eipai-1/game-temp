use crate::camera;

#[derive(Debug)]
pub struct Benchmark {
    pub is_active: bool,
    //first_active_flag: bool,
    pub sample_count: u32,
    total_sample_count: u32,
    pub has_output: bool,

    //secs as f64
    period: f64,
    period_dt: f64,

    total_fps: f64,
    pub avg_fps: f64,
    pub max_fps: f64,
    pub min_fps: f64,
}

impl Benchmark {
    pub fn new() -> Self {
        let is_active = false;
        let sample_count = 0;
        let total_sample_count = 10;
        let avg_fps = 0.0;
        let max_fps = f64::INFINITY;
        let min_fps = 0.0;
        let total_fps = 0.0;
        let period = 1.0;
        let period_dt = 0.0;
        let has_output = false;
        //let first_active_flag = true;
        Self {
            sample_count,
            avg_fps,
            max_fps,
            min_fps,
            is_active,
            period,
            period_dt,
            total_sample_count,
            total_fps,
            has_output,
            //first_active_flag,
        }
    }

    pub fn start(&mut self, camera: &mut camera::Camera) {
        if self.is_active == false {
            self.reset();
            self.is_active = true;
            camera.reset();
        }
    }

    fn reset(&mut self) {
        self.is_active = false;
        self.has_output = false;
        self.sample_count = 0;
        self.total_fps = 0.0;
        self.max_fps = f64::INFINITY;
        self.min_fps = 0.0;
    }

    pub fn update(&mut self, dt: f64) {
        if self.is_active {
            //println!("start benchmarking");
            if self.sample_count < self.total_sample_count {
                //周期采样
                if self.period_dt >= self.period {
                    let fps = 1.0 / dt;

                    if fps < self.max_fps {
                        self.max_fps = fps;
                    }
                    if fps > self.min_fps {
                        self.min_fps = fps;
                    }
                    self.sample_count += 1;

                    self.total_fps += fps;

                    self.period_dt = 0.0;
                }
                self.period_dt += dt;
            } else if !self.has_output {
                self.avg_fps = self.total_fps / self.total_sample_count as f64;
                self.has_output = true;
                self.is_active = false;
                //println!("{:#?}", self);
            }
        }
    }
}
