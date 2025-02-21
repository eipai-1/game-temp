use std::{iter, sync::Arc};

use pollster::FutureExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

mod basic_config;
mod control;

struct State {
    basic_config: basic_config::BasicConfig,
    window: Arc<Window>,
}

impl State {
    async fn new(window: Window) -> Self {
        let window = Arc::new(window);

        let basic_config = basic_config::BasicConfig::new(Arc::clone(&window)).await;

        Self {
            basic_config,
            window,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.basic_config.size = new_size;
            self.basic_config.config.width = new_size.width;
            self.basic_config.config.height = new_size.height;
            self.basic_config
                .surface
                .configure(&self.basic_config.device, &self.basic_config.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.basic_config.clear_color = wgpu::Color {
                    r: position.x / self.basic_config.size.width as f64,
                    g: position.y / self.basic_config.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.basic_config.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.basic_config
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        //这个大括号是必须的
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.basic_config.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.basic_config.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App {
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

//窗口事件处理
impl ApplicationHandler for App {
    //Windows平台中，仅初始化会调用 -ai
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title("game-temp")
            .with_inner_size(PhysicalSize::new(800, 600));
        let window = event_loop.create_window(window_attributes).unwrap();

        //异步函数这里不是很懂
        self.state = Some(State::new(window).block_on());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = self.state.as_mut() {
            //如果为真则代表为输入事件，且此方法会处理输入事件。此时则完成处理，不需要在继续处理
            //否则继续处理
            if state.input(&event) {
                return;
            }
        }
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.resize(state.basic_config.size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            event_loop.exit();
                        }

                        // This happens when the a frame takes too long to present
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }

                        //wgpu-new
                        //新版wgpu新增的
                        Err(wgpu::SurfaceError::Other) => {
                            log::warn!("Surface error: other")
                        }
                    }
                    //循环调用从而不断重绘
                    state.window.request_redraw();
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(physical_size);
                }
            }
            _ => {}
        }
    }
}
pub fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}
