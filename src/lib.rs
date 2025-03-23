use crate::egui_tools::EguiRenderer;
use egui_wgpu::ScreenDescriptor;
use instant::Instant;
use pollster::FutureExt;
use std::{iter, sync::Arc};
use wgpu::*;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

mod basic_config;
mod benchmark;
pub mod camera;
mod control;
mod egui_tools;
mod game_config;
mod physics;
pub mod realm;
mod texture;
pub mod voxel_collision;
pub mod voxel_collision_example;

struct State {
    basic_config: basic_config::BasicConfig,
    window: Arc<Window>,
    render_pipeline: RenderPipeline,
    //好像没用？
    //diffuse_texture: texture::Texture,
    diffuse_bind_group: BindGroup,

    dt: f64,
    last_render_time: instant::Instant,

    depth_texture: texture::Texture,

    realm: realm::Realm,
    wf_render_pipeline: RenderPipeline,

    game_config: game_config::GameConfig,
    benchmark: benchmark::Benchmark,

    egui_renderer: EguiRenderer,
    scale_factor: f32,

    player: physics::PlayerEntity,
    control: control::Control,
}

impl State {
    async fn new(window: Window) -> Self {
        let window = Arc::new(window);

        //基础配置
        let basic_config = basic_config::BasicConfig::new(Arc::clone(&window)).await;

        let depth_texture = texture::Texture::create_depth_texture(
            &basic_config.device,
            &basic_config.config,
            "Depth texture",
        );

        let realm = realm::Realm::new(&basic_config.device);

        let game_config = game_config::GameConfig::new();

        let player = physics::PlayerEntity::new(&basic_config, &realm, &game_config);

        //摄像机创建完成

        //开始创建diffuse_bind_group
        //let diffuse_bytes = include_bytes!("../res/tile_map.png");
        let diffuse_texture =
            texture::Texture::load_blocks("res/texture", &basic_config.device, &basic_config.queue)
                .unwrap();

        let texture_bind_group_layout =
            basic_config
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Texture bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2Array,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let diffuse_bind_group =
            basic_config
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("First diffuse bind group"),
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                    ],
                });
        //创建diffuse_bind_group完成

        //以下是创建 render_pipeline 和 buffer
        let shader = basic_config
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: Some("First Shader"),
                source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        //let num_indices = realm::INDICES.len() as u32;

        let render_pipeline_layout =
            basic_config
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("First render pipeline layout"),
                    bind_group_layouts: &[
                        &player.camera_bind_group_layout,
                        &texture_bind_group_layout,
                        &realm.render_res.block_materials_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            basic_config
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("First render pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[realm::Vertex::desc(), realm::Instance::desc()],
                        compilation_options: PipelineCompilationOptions::default(),
                    },
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Cw,
                        cull_mode: Some(Face::Back),
                        unclipped_depth: false,
                        polygon_mode: PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: texture::Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(ColorTargetState {
                            format: basic_config.config.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })],
                        compilation_options: PipelineCompilationOptions::default(),
                    }),
                    multiview: None,
                    cache: None,
                });
        //render_pipeline和buffer创建完成

        //线框
        let wf_shader = basic_config
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: Some("Wireframe Shader"),
                source: ShaderSource::Wgsl(include_str!("wf_shader.wgsl").into()),
            });

        let wf_render_pipeline =
            basic_config
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("Wireframe render pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: VertexState {
                        module: &wf_shader,
                        entry_point: "vs_main",
                        buffers: &[realm::WireframeVertex::desc()],
                        compilation_options: PipelineCompilationOptions::default(),
                    },
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Cw,
                        cull_mode: Some(Face::Back),
                        unclipped_depth: false,
                        polygon_mode: PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: texture::Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Less,
                        stencil: StencilState::default(),
                        bias: DepthBiasState::default(),
                    }),
                    multisample: MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(FragmentState {
                        module: &wf_shader,
                        entry_point: "fs_main",
                        targets: &[Some(ColorTargetState {
                            format: basic_config.config.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })],
                        compilation_options: PipelineCompilationOptions::default(),
                    }),
                    multiview: None,
                    cache: None,
                });
        //线框创建完成

        let dt: f64 = 0.001;
        let last_render_time = instant::Instant::now();

        let benchmark = benchmark::Benchmark::new();

        let egui_renderer = EguiRenderer::new(
            &basic_config.device,
            basic_config.config.format,
            None,
            1,
            &*window,
        );

        let scale_factor: f32 = 1.0;

        let control = control::Control::new(&player.camera);

        Self {
            basic_config,
            window,
            render_pipeline,
            //diffuse_texture,
            diffuse_bind_group,

            player,

            dt,
            last_render_time,

            depth_texture,

            realm,
            wf_render_pipeline,

            game_config,

            benchmark,
            egui_renderer,
            scale_factor,

            control,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.basic_config.size = new_size;
            self.basic_config.config.width = new_size.width;
            self.basic_config.config.height = new_size.height;
            self.basic_config
                .surface
                .configure(&self.basic_config.device, &self.basic_config.config);
            self.player.camera_controller.center_x = new_size.width / 2;
            self.player.camera_controller.center_y = new_size.height / 2;
            self.player
                .projection
                .resize(new_size.width, new_size.height);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.basic_config.device,
                &self.basic_config.config,
                "depth_texture",
            );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        if self.player.camera_controller.is_fov {
            self.window
                .as_ref()
                .set_cursor_position(PhysicalPosition::new(
                    self.player.camera_controller.center_x,
                    self.player.camera_controller.center_y,
                ))
                .unwrap();
        }
        self.control.process_events(
            &mut self.player,
            event,
            &mut self.realm,
            &self.basic_config.queue,
            &mut self.game_config,
        )
    }

    fn update(&mut self) {
        self.player.update(self.dt as f32, &mut self.realm.data);

        self.realm
            .update(&self.player.entity.position, &self.basic_config.device);

        self.basic_config.queue.write_buffer(
            &self.player.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.player.camera_uniform]),
        );
        self.basic_config.queue.write_buffer(
            &self.realm.render_res.wf_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.realm.data.wf_uniform),
        );

        self.benchmark.update(self.dt);
        self.egui_renderer.update(self.dt);
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.basic_config.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder =
            self.basic_config
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        //这个大括号是必须的
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.basic_config.clear_color),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.player.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(2, &self.realm.render_res.block_materials_bind_group, &[]);
            //for block in self.realm.data.all_block.iter().skip(1) {
            //    if self.realm.data.instances[block.block_type as usize].len() == 0 {
            //        continue;
            //    }
            //    render_pass.set_vertex_buffer(
            //        0,
            //        self.realm.render_res.block_vertex_buffers[block.block_type as usize].slice(..),
            //    );
            //    render_pass.set_vertex_buffer(
            //        1,
            //        self.realm.render_res.instance_buffers[block.block_type as usize].slice(..),
            //    );
            //    render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            //    render_pass.draw_indexed(
            //        0..realm::INDICES.len() as u32,
            //        0,
            //        0..self.realm.data.instances[block.block_type as usize].len() as _,
            //    );
            //}
            render_pass.set_vertex_buffer(0, self.realm.render_res.block_vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.realm.render_res.block_index_buffer.slice(..),
                IndexFormat::Uint16,
            );
            for (coord, chunk) in self.realm.data.chunk_map.iter() {
                render_pass
                    .set_vertex_buffer(1, self.realm.render_res.instance_buffers[&coord].slice(..));
                render_pass.draw_indexed(
                    0..realm::INDICES.len() as _,
                    0,
                    0..chunk.offset_top as u32,
                );
            }

            //绘制线框
            if self.realm.data.is_wf_visible {
                render_pass.set_pipeline(&self.wf_render_pipeline);
                render_pass.set_bind_group(0, &self.player.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.realm.render_res.wf_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.realm.render_res.wf_index_buffer.slice(..),
                    IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..realm::WIREFRAME_INDCIES.len() as u32, 0, 0..1);
            }

            // 结束当前渲染通道，这里很重要！
        } // 这里render_pass会被drop，自动结束

        // 现在调用debug_window，此时编码器没有被锁定
        self.debug_window(&mut encoder, &output);

        self.basic_config.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn debug_window(&mut self, encoder: &mut CommandEncoder, output: &SurfaceTexture) {
        if self.game_config.is_debug_window_open {
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [
                    self.basic_config.config.width,
                    self.basic_config.config.height,
                ],
                pixels_per_point: self.window.as_ref().scale_factor() as f32 * self.scale_factor,
            };

            let surface_view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            //let mut encoder =
            //    self.basic_config
            //        .device
            //        .create_command_encoder(&CommandEncoderDescriptor {
            //            label: Some("temp encoder"),
            //        });

            let window = self.window.as_ref();
            {
                self.egui_renderer.begin_frame(window);

                egui::Window::new("Debug window")
                    .resizable(true)
                    .vscroll(true)
                    .default_open(true)
                    .default_size((200.0, 300.0))
                    .show(self.egui_renderer.context(), |ui| {
                        ui.label(format!("FPS:{}", self.egui_renderer.fps as u32));
                        ui.label(format!("x:{}", self.player.entity.position.x));
                        ui.label(format!("y:{}", self.player.entity.position.y));
                        ui.label(format!("z:{}", self.player.entity.position.z));
                        ui.label(format!("chunk_map.len:{}", self.realm.data.chunk_map.len()));

                        if let Some(selected_block) = self.player.camera_controller.selected_block {
                            ui.label(format!(
                                "selected block:({},{},{}):({:?})",
                                selected_block.x,
                                selected_block.y,
                                selected_block.z,
                                self.realm.data.get_block(selected_block).tp
                            ));
                        }
                        if let Some(pre_selected_block) =
                            self.player.camera_controller.pre_selected_block
                        {
                            ui.label(format!(
                                "pre_selected block:({},{},{})",
                                pre_selected_block.x, pre_selected_block.y, pre_selected_block.z
                            ));
                        }

                        ui.label(format!("is_grounded:{}", self.player.entity.is_grounded));
                        //ui.label(format!("is_collided:{}", self.player.is_collided));
                        ui.label(format!("is_testing:{}", self.player.entity.is_testing));
                        ui.label(format!("velocity:{:?}", self.player.entity.velocity));

                        //if ui.button("debug print").clicked() {}

                        if self.game_config.get_max_fps() == 0 {
                            if ui.button("set max FPS to 60").clicked() {
                                self.game_config.set_max_fps(60);
                            }
                        } else {
                            if ui.button("set max FPS unlimited").clicked() {
                                self.game_config.set_max_fps(0);
                            }
                        }

                        if ui.button("start benchmark").clicked() {
                            self.benchmark.start(&mut self.player.camera);
                        }
                        ui.separator();

                        if self.benchmark.is_active {
                            ui.label("Running benchmark");
                        }

                        if self.benchmark.has_output {
                            ui.label(format!(
                                "Benchmark Result: Avg FPS:{:.2}, sample_count:{} ",
                                self.benchmark.avg_fps, self.benchmark.sample_count
                            ));
                        }
                    });

                self.egui_renderer.end_frame_and_draw(
                    &self.basic_config.device,
                    &self.basic_config.queue,
                    encoder,
                    window,
                    &surface_view,
                    screen_descriptor,
                );
            }
        }
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
            .with_inner_size(PhysicalSize::new(1200, 800));
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
            state.input(&event);
            state
                .egui_renderer
                .handle_input(state.window.as_ref(), &event);
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
                    let now = Instant::now();
                    state.dt = now.duration_since(state.last_render_time).as_secs_f64();
                    state.last_render_time = now;

                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                            state.resize(state.basic_config.size)
                        }
                        // The system is out of memory, we should probably quit
                        Err(SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            event_loop.exit();
                        }

                        // This happens when the a frame takes too long to present
                        Err(SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                    }

                    if state.game_config.get_max_fps() != 0 {
                        let elapsed = state.last_render_time.elapsed();
                        if elapsed < state.game_config.get_frame_duration() {
                            state
                                .game_config
                                .sleeper
                                .sleep(state.game_config.get_frame_duration() - elapsed);
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
