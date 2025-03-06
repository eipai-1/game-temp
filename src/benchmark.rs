use crate::camera;

#[derive(Debug)]
pub struct Benchmark {
    is_active: bool,
    sample_count: u32,
    total_sample_count: u32,
    has_output: bool,

    //secs as f64
    period: f64,
    period_dt: f64,

    total_fps: f64,
    avg_fps: f64,
    max_fps: f64,
    min_fps: f64,
}

impl Benchmark {
    pub fn new(is_active: bool) -> Self {
        let sample_count = 0;
        let total_sample_count = 10;
        let avg_fps = 0.0;
        let max_fps = f64::INFINITY;
        let min_fps = 0.0;
        let total_fps = 0.0;
        let period = 1.0;
        let period_dt = 0.0;
        let has_output = false;
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
        }
    }

    pub fn update(&mut self, dt: f64) {
        if self.is_active {
            //println!("start benchmarking");
            if self.sample_count < self.total_sample_count {
                //周期采样
                if self.period_dt < self.period {
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
                println!("{:#?}", self);
            }
        }
    }
}
