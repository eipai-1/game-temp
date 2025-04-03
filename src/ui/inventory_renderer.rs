use crate::realm::{self};
use crate::ui::block_renderer::BlockRenderer;
use crate::ui::{UIInstance, UIVertex};
use wgpu::{util::DeviceExt, *};

pub const HOTBAR_INDEX: &[u16] = &[0, 1, 2, 0, 2, 3];

pub const SLOT_SIZE: f32 = 64.0;
pub const SLOT_SPACING: f32 = 10.0;
pub const SLOTS_PER_ROW: u32 = 10;
pub const SLOTS_PER_COLUMN: u32 = 4;
pub const IV_WIDTH: f32 =
    SLOT_SIZE * SLOTS_PER_ROW as f32 + SLOT_SPACING * (SLOTS_PER_ROW + 1) as f32;
pub const IV_HEIGHT: f32 =
    SLOT_SIZE * SLOTS_PER_COLUMN as f32 + SLOT_SPACING * (SLOTS_PER_COLUMN + 1) as f32;
pub const HOTBAR_LEFT: f32 = 30.0;
pub const HOTBAR_TOP: f32 = 30.0;
const SELECTED_FRAME_SIZE: f32 = 8.0;

//hb 代表hotbar
//shb 代表selected hotbar
//iv 代表inventory
pub struct InventoryRenderer {
    pub is_dragging: bool,
    pub dragging_instance: Option<realm::Instance>,
    pub dragging_instance_buffer: Option<Buffer>,

    hb_vertices: Vec<UIVertex>,
    hb_instances: Vec<UIInstance>,
    hb_vertex_buffer: Buffer,
    hb_instance_buffer: Buffer,
    hb_index_buffer: Buffer,
    shb_index: i32,
    shb_vertices: Vec<UIVertex>,
    shb_instances: Vec<UIInstance>,
    shb_vertex_buffer: Buffer,
    shb_instance_buffer: Buffer,
    pub render_pipeline: RenderPipeline,
    iv_vertices: Vec<UIVertex>,
    iv_instances: Vec<UIInstance>,
    iv_slot_instances: Vec<UIInstance>,
    iv_vertex_buffer: Buffer,
    iv_instance_buffer: Buffer,
    iv_slot_instance_buffer: Buffer,
    pub block_renderer: BlockRenderer,
}

impl InventoryRenderer {
    pub fn new(
        device: &Device,
        format: TextureFormat,
        screen_size_uniform_bind_group_layout: &BindGroupLayout,
        physical_size: winit::dpi::PhysicalSize<u32>,
        block_renderer: BlockRenderer,
    ) -> Self {
        let (hb_vertices, hb_instances) = Self::create_hb_vetices_instances();
        let (iv_vertices, iv_instances) = Self::create_iv_vertices_instances(physical_size);
        let iv_slot_instances = Self::create_iv_slot_instances(physical_size);
        let (shb_vertices, shb_instances) = Self::create_shb_vertices_instances();

        let shb_index = 0;

        let hb_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Vertex Buffer"),
            contents: bytemuck::cast_slice(&hb_vertices),
            usage: BufferUsages::VERTEX,
        });

        let hb_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Instance Buffer"),
            contents: bytemuck::cast_slice(&hb_instances),
            usage: BufferUsages::VERTEX,
        });

        let hb_index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Index Buffer"),
            contents: bytemuck::cast_slice(&HOTBAR_INDEX),
            usage: BufferUsages::INDEX,
        });

        let shb_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Vertex Buffer"),
            contents: bytemuck::cast_slice(&shb_vertices),
            usage: BufferUsages::VERTEX,
        });

        let shb_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Instance Buffer"),
            contents: bytemuck::cast_slice(&shb_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let iv_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Vertex Buffer"),
            contents: bytemuck::cast_slice(&iv_vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let iv_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Instance Buffer"),
            contents: bytemuck::cast_slice(&iv_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let iv_slot_instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Slot Instance Buffer"),
            contents: bytemuck::cast_slice(&iv_slot_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Inventory Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui_instance_shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&screen_size_uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("UI render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[UIVertex::desc(), UIInstance::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        InventoryRenderer {
            hb_vertex_buffer,
            hb_instance_buffer,
            hb_index_buffer,
            hb_instances,
            hb_vertices,
            render_pipeline,
            iv_vertices,
            iv_instances,
            iv_slot_instances,
            iv_vertex_buffer,
            iv_instance_buffer,
            iv_slot_instance_buffer,
            shb_instances,
            shb_vertices,
            shb_vertex_buffer,
            shb_instance_buffer,
            shb_index,
            block_renderer,
            is_dragging: false,
            dragging_instance: None,
            dragging_instance_buffer: None,
        }
    }

    pub fn draw_hotbar(
        &self,
        screen_size_uniform_bind_group: &BindGroup,
        render_pass: &mut RenderPass,
    ) {
        render_pass.set_bind_group(0, screen_size_uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.shb_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.shb_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.hb_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..HOTBAR_INDEX.len() as _,
            0,
            0..self.shb_instances.len() as _,
        );

        render_pass.set_vertex_buffer(0, self.hb_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.hb_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.hb_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..HOTBAR_INDEX.len() as _,
            0,
            0..self.hb_instances.len() as _,
        );
    }

    pub fn draw_inventory(&self, render_pass: &mut RenderPass) {
        render_pass.set_vertex_buffer(0, self.iv_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.iv_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.hb_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..HOTBAR_INDEX.len() as _,
            0,
            0..self.iv_instances.len() as _,
        );
        render_pass.set_vertex_buffer(0, self.hb_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.iv_slot_instance_buffer.slice(..));
        render_pass.set_index_buffer(self.hb_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..HOTBAR_INDEX.len() as _,
            0,
            0..self.iv_slot_instances.len() as _,
        );
    }

    pub fn resize(&mut self, queue: &Queue, physical_size: winit::dpi::PhysicalSize<u32>) {
        // 重新计算物品栏的位置
        let (iv_vertices, iv_instances) = Self::create_iv_vertices_instances(physical_size);
        let iv_slot_instances = Self::create_iv_slot_instances(physical_size);

        // 更新缓冲区
        queue.write_buffer(
            &self.iv_vertex_buffer,
            0,
            bytemuck::cast_slice(&iv_vertices),
        );

        queue.write_buffer(
            &self.iv_instance_buffer,
            0,
            bytemuck::cast_slice(&iv_instances),
        );

        queue.write_buffer(
            &self.iv_slot_instance_buffer,
            0,
            bytemuck::cast_slice(&iv_slot_instances),
        );

        // 更新实例数据
        self.iv_vertices = iv_vertices;
        self.iv_instances = iv_instances;
        self.iv_slot_instances = iv_slot_instances;
    }

    pub fn update_shb(&mut self, offset: i32, queue: &Queue) {
        let mut x = self.shb_index + offset;
        x += SLOTS_PER_ROW as i32;
        x %= SLOTS_PER_ROW as i32;
        self.shb_index = x;
        let shb_instances = vec![UIInstance::new(x as f32 * (SLOT_SIZE + SLOT_SPACING), 0.0,); 1];

        // 更新选中物品栏的实例数据
        queue.write_buffer(
            &self.shb_instance_buffer,
            0,
            bytemuck::cast_slice(&shb_instances),
        );
    }

    fn create_hb_vetices_instances() -> (Vec<UIVertex>, Vec<UIInstance>) {
        let mut hb_vertices = Vec::new();
        let left = 0.0;
        let right = SLOT_SIZE;
        let top = 0.0;
        let bottom = SLOT_SIZE;
        hb_vertices.push(UIVertex {
            position: [left, top],
            color: [0.2, 0.4, 0.3, 0.7],
        });
        hb_vertices.push(UIVertex {
            position: [right, top],
            color: [0.2, 0.4, 0.3, 0.7],
        });
        hb_vertices.push(UIVertex {
            position: [right, bottom],
            color: [0.2, 0.4, 0.3, 0.7],
        });
        hb_vertices.push(UIVertex {
            position: [left, bottom],
            color: [0.2, 0.4, 0.3, 0.7],
        });

        //距离屏幕左边的距离
        let mut hb_instances: Vec<UIInstance> = Vec::new();
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT, HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + SLOT_SIZE + SLOT_SPACING, HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 2.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 3.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 4.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 5.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 6.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 7.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 8.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        hb_instances.push(UIInstance {
            position: [HOTBAR_LEFT + 9.0 * (SLOT_SIZE + SLOT_SPACING), HOTBAR_TOP],
            _padding: [0.0, 0.0],
        });
        (hb_vertices, hb_instances)
    }

    fn create_iv_vertices_instances(
        physical_size: winit::dpi::PhysicalSize<u32>,
    ) -> (Vec<UIVertex>, Vec<UIInstance>) {
        let mut iv_vertices: Vec<UIVertex> = Vec::new();

        let color = glyphon::Color::rgb(48, 174, 149);

        let iv_bg_color = [
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            1.0,
        ];
        let left = physical_size.width as f32 / 2.0 - IV_WIDTH / 2.0;
        let right = physical_size.width as f32 / 2.0 + IV_WIDTH / 2.0;
        let top = physical_size.height as f32 / 2.0 - IV_HEIGHT / 2.0;
        let bottom = physical_size.height as f32 / 2.0 + IV_HEIGHT / 2.0;
        iv_vertices.push(UIVertex {
            position: [left, top],
            color: iv_bg_color,
        });
        iv_vertices.push(UIVertex {
            position: [right, top],
            color: iv_bg_color,
        });
        iv_vertices.push(UIVertex {
            position: [right, bottom],
            color: iv_bg_color,
        });
        iv_vertices.push(UIVertex {
            position: [left, bottom],
            color: iv_bg_color,
        });
        let mut iv_instances: Vec<UIInstance> = Vec::new();
        iv_instances.push(UIInstance::new(0.0, 0.0));
        (iv_vertices, iv_instances)
    }

    fn create_iv_slot_instances(physical_size: winit::dpi::PhysicalSize<u32>) -> Vec<UIInstance> {
        let iv_slot_left = physical_size.width as f32 / 2.0 - IV_WIDTH / 2.0 + SLOT_SPACING;
        let iv_slot_top = physical_size.height as f32 / 2.0 - IV_HEIGHT / 2.0 + SLOT_SPACING;

        let mut iv_slot_instances: Vec<UIInstance> = Vec::new();
        for i in 0..SLOTS_PER_ROW {
            for j in 0..SLOTS_PER_COLUMN + 1 {
                iv_slot_instances.push(UIInstance::new(
                    iv_slot_left + (SLOT_SIZE + SLOT_SPACING) * i as f32,
                    iv_slot_top + (SLOT_SIZE + SLOT_SPACING) * j as f32,
                ));
            }
        }

        iv_slot_instances
    }

    fn create_shb_vertices_instances() -> (Vec<UIVertex>, Vec<UIInstance>) {
        let mut shb_vertices = Vec::new();

        let color_rgb = glyphon::Color::rgb(115, 254, 189);
        let color = [
            color_rgb.r() as f32 / 255.0,
            color_rgb.g() as f32 / 255.0,
            color_rgb.b() as f32 / 255.0,
            1.0,
        ];
        shb_vertices.push(UIVertex {
            position: [
                HOTBAR_LEFT - SELECTED_FRAME_SIZE,
                HOTBAR_TOP - SELECTED_FRAME_SIZE,
            ],
            color: color,
        });
        shb_vertices.push(UIVertex {
            position: [
                HOTBAR_LEFT + SLOT_SIZE + SELECTED_FRAME_SIZE,
                HOTBAR_TOP - SELECTED_FRAME_SIZE,
            ],
            color: color,
        });
        shb_vertices.push(UIVertex {
            position: [
                HOTBAR_LEFT + SLOT_SIZE + SELECTED_FRAME_SIZE,
                HOTBAR_TOP + SLOT_SIZE + SELECTED_FRAME_SIZE,
            ],
            color: color,
        });
        shb_vertices.push(UIVertex {
            position: [
                HOTBAR_LEFT - SELECTED_FRAME_SIZE,
                HOTBAR_TOP + SLOT_SIZE + SELECTED_FRAME_SIZE,
            ],
            color: color,
        });
        let mut shb_instances = Vec::new();
        shb_instances.push(UIInstance::new(0.0, 0.0));
        (shb_vertices, shb_instances)
    }

    pub fn create_dragging_instance(&mut self, tp: u32, x: f32, y: f32, device: &Device) {
        let instance = realm::Instance {
            position: [x, y, 0.0],
            block_type: tp,
        };

        self.dragging_instance = Some(instance);

        self.dragging_instance_buffer =
            Some(device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some("Dragging Instance Buffer"),
                contents: bytemuck::cast_slice(&[instance]),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }));
    }

    pub fn update_dragging_instance(&self, x: f32, y: f32, queue: &Queue) {
        let mut instance = self.dragging_instance.unwrap();
        instance.position[0] = x;
        instance.position[1] = y;
        queue.write_buffer(
            self.dragging_instance_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[instance]),
        );
    }
}
pub fn get_selected_slot(
    mut x: f32,
    mut y: f32,
    physical_size: winit::dpi::PhysicalSize<u32>,
) -> (u32, u32) {
    x -= physical_size.width as f32 / 2.0 - IV_WIDTH / 2.0 + SLOT_SPACING;
    x /= SLOT_SIZE + SLOT_SPACING;
    y -= physical_size.height as f32 / 2.0 - IV_HEIGHT / 2.0 + SLOT_SPACING;
    y /= SLOT_SIZE + SLOT_SPACING;
    (x as u32, y as u32)
}
