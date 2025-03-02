use instant::Instant;
use pollster::FutureExt;
use std::{iter, sync::Arc};
use util::DeviceExt;
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
pub mod camera;
mod game_config;
pub mod realm;
mod texture;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}

struct State {
    basic_config: basic_config::BasicConfig,
    window: Arc<Window>,
    render_pipeline: RenderPipeline,
    index_buffer: Buffer,

    //好像没用？
    //diffuse_texture: texture::Texture,
    diffuse_bind_group: BindGroup,

    //摄像机相关
    camera: camera::Camera,
    projection: camera::Projection,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    camera_controller: camera::CameraController,

    dt: f64,
    last_render_time: instant::Instant,

    depth_texture: texture::Texture,

    realm: realm::Realm,
    wf_render_pipeline: RenderPipeline,

    game_config: game_config::GameConfig,
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

        //创建摄像机
        let camera = camera::Camera::new((1.0, 8.0, 1.0), cgmath::Deg(90.0), cgmath::Deg(-45.0));
        let projection = camera::Projection::new(
            basic_config.config.width,
            basic_config.config.height,
            cgmath::Deg(45.0),
            0.1,
            100.0,
        );
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer =
            basic_config
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("First camera buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout =
            basic_config
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::VERTEX,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let camera_bind_group = basic_config
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera bind group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: realm.render_res.wf_uniform_buffer.as_entire_binding(),
                    },
                ],
            });

        let camera_controller = camera::CameraController::new(
            2.0,
            basic_config.config.width / 2,
            basic_config.config.height / 2,
        );
        //摄像机创建完成

        //开始创建diffuse_bind_group
        let diffuse_bytes = include_bytes!("../res/tile_map.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &basic_config.device,
            &basic_config.queue,
            diffuse_bytes,
            "tile_map",
        )
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
                                view_dimension: TextureViewDimension::D2,
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

        let index_buffer = basic_config
            .device
            .create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(realm::INDICES),
                usage: BufferUsages::INDEX,
            });

        //let num_indices = realm::INDICES.len() as u32;

        let render_pipeline_layout =
            basic_config
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("First render pipeline layout"),
                    bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
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
                        entry_point: Some("vs_main"),
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
                        entry_point: Some("fs_main"),
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
                        entry_point: Some("vs_main"),
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
                        entry_point: Some("fs_main"),
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

        let game_config = game_config::GameConfig::new();

        let dt: f64 = 0.001;
        let last_render_time = instant::Instant::now();

        Self {
            basic_config,
            window,
            render_pipeline,
            index_buffer,
            //diffuse_texture,
            diffuse_bind_group,

            camera,
            projection,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,

            dt,
            last_render_time,

            depth_texture,

            realm,
            wf_render_pipeline,

            game_config,
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
            self.camera_controller.center_x = new_size.width / 2;
            self.camera_controller.center_y = new_size.height / 2;
            self.projection.resize(new_size.width, new_size.height);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.basic_config.device,
                &self.basic_config.config,
                "depth_texture",
            );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        if self.camera_controller.is_fov {
            self.window
                .as_ref()
                .set_cursor_position(PhysicalPosition::new(
                    self.camera_controller.center_x,
                    self.camera_controller.center_y,
                ))
                .unwrap();
        }
        self.camera_controller
            .process_events(event, &mut self.camera, self.dt as f32)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(
            &mut self.camera,
            self.dt as f32,
            &mut self.realm.data,
        );
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);

        self.realm
            .update(&self.camera.position, &self.basic_config.device);

        self.basic_config.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.basic_config.queue.write_buffer(
            &self.realm.render_res.wf_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.realm.data.wf_uniform),
        );
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
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);

            for block in self.realm.data.all_block.iter().skip(1) {
                if self.realm.data.instances[block.block_type as usize].len() == 0 {
                    continue;
                }
                render_pass.set_vertex_buffer(
                    0,
                    self.realm.render_res.block_vertex_buffers[block.block_type as usize].slice(..),
                );
                render_pass.set_vertex_buffer(
                    1,
                    self.realm.render_res.instance_buffers[block.block_type as usize].slice(..),
                );
                render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(
                    0..realm::INDICES.len() as u32,
                    0,
                    0..self.realm.data.instances[block.block_type as usize].len() as _,
                );
            }

            //绘制线框
            if self.realm.data.is_wf_visible {
                render_pass.set_pipeline(&self.wf_render_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.diffuse_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.realm.render_res.wf_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.realm.render_res.wf_index_buffer.slice(..),
                    IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..realm::WIREFRAME_INDCIES.len() as u32, 0, 0..1);
            }
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
            state.input(&event);
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

                        //wgpu-new
                        //新版wgpu新增的
                        Err(SurfaceError::Other) => {
                            log::warn!("Surface error: other")
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
