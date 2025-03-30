use glyphon::{Color, TextArea, TextBounds};
use std::vec;

use wgpu::{util::DeviceExt, *};

use crate::realm;

mod block_render;
pub mod ui_text_renderer;

const CURSOR_HALF_SIZE: f32 = 12.0;
const CURSOR_HALF_LENGTH: f32 = 2.0;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UIVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UIUniform {
    screen_size: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 2],
    _padding: [f32; 2],
}

impl Instance {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 5,
            }],
        }
    }
}

const CURSOR_INDEX: &[u16] = &[0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7];
const INVENTORY_INDEX: &[u16] = &[0, 1, 2, 0, 2, 3];

impl UIVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}

struct InventoryRenderer {
    vertices: Vec<UIVertex>,
    instances: Vec<Instance>,
    vertex_buffer: Buffer,
    instance_buffer: Buffer,
    index_buffer: Buffer,
    render_pipeline: RenderPipeline,
}

impl InventoryRenderer {
    fn new(
        device: &Device,
        format: TextureFormat,
        screen_size_uniform_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let mut vertices = Vec::new();
        let slot_size = 64.0;
        let left = 0.0;
        let right = slot_size;
        let top = 0.0;
        let bottom = slot_size;
        vertices.push(UIVertex {
            position: [left, top],
            color: [0.2, 0.4, 0.3, 0.7],
        });
        vertices.push(UIVertex {
            position: [right, top],
            color: [0.2, 0.4, 0.3, 0.7],
        });
        vertices.push(UIVertex {
            position: [right, bottom],
            color: [0.2, 0.4, 0.3, 0.7],
        });
        vertices.push(UIVertex {
            position: [left, bottom],
            color: [0.2, 0.4, 0.3, 0.7],
        });

        let inventory_slot_spacing = 10.0f32;

        //距离屏幕左边的距离
        let inventory_left = 30.0f32;
        //距离屏幕上边的距离
        let inventory_top = 30.0f32;

        let mut instances: Vec<Instance> = Vec::new();
        instances.push(Instance {
            position: [inventory_left, inventory_top],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + slot_size + inventory_slot_spacing,
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 2.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 3.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 4.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 5.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 6.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 7.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 8.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });
        instances.push(Instance {
            position: [
                inventory_left + 9.0 * (slot_size + inventory_slot_spacing),
                inventory_top,
            ],
            _padding: [0.0, 0.0],
        });

        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Inventory Index Buffer"),
            contents: bytemuck::cast_slice(&INVENTORY_INDEX),
            usage: BufferUsages::INDEX,
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Inventory Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("inventory_shader.wgsl").into()),
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
                buffers: &[UIVertex::desc(), Instance::desc()],
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
            vertex_buffer,
            instance_buffer,
            index_buffer,
            instances,
            vertices,
            render_pipeline,
        }
    }
}

pub struct UI {
    cursor_vertex_buffer: Buffer,
    cursor_index_buffer: Buffer,
    cursor_vertices: Vec<UIVertex>,
    pub ui_text_renderer: ui_text_renderer::UITextRenderer,
    screen_size_uniform_buffer: Buffer,
    screen_size_uniform_bind_group: BindGroup,
    instance_buffer: Buffer,
    instances: Vec<Instance>,
    inventory_renderer: InventoryRenderer,
    block_renderer: block_render::BlockRenderer,
}

impl UI {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        scale_factor: f32,
        physical_size: winit::dpi::PhysicalSize<u32>,
        block_materials_bind_group_layout: &BindGroupLayout,
        texture_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let ui_text_renderer = ui_text_renderer::UITextRenderer::new(
            device,
            queue,
            format,
            scale_factor,
            physical_size,
        );

        let instances = vec![Instance {
            position: [0.0, 0.0],
            _padding: [0.0, 0.0],
        }];

        let instance_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: BufferUsages::VERTEX,
        });

        let screen_size_uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("UI Uniform Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_size_uniform_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("UI Uniform Buffer"),
            contents: bytemuck::cast_slice(&[UIUniform {
                screen_size: [physical_size.width as f32, physical_size.height as f32],
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let screen_size_uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("UI Uniform Bind Group"),
            layout: &screen_size_uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: screen_size_uniform_buffer.as_entire_binding(),
            }],
        });

        let cursor_vertices = Self::generate_cursor_vertices(physical_size);

        let cursor_vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Cursor Vertex Buffer"),
            contents: bytemuck::cast_slice(&cursor_vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        let cursor_index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Cursor Index Buffer"),
            contents: bytemuck::cast_slice(&CURSOR_INDEX),
            usage: BufferUsages::INDEX,
        });

        let inventory_renderer =
            InventoryRenderer::new(device, format, &screen_size_uniform_bind_group_layout);

        let block_renderer = block_render::BlockRenderer::new(
            device,
            format,
            &screen_size_uniform_bind_group_layout,
            block_materials_bind_group_layout,
            texture_bind_group_layout,
        );

        Self {
            cursor_vertex_buffer,
            cursor_vertices,
            ui_text_renderer,
            cursor_index_buffer,
            screen_size_uniform_buffer,
            screen_size_uniform_bind_group,
            inventory_renderer,
            instance_buffer,
            instances,
            block_renderer,
        }
    }

    fn generate_cursor_vertices(physical_size: winit::dpi::PhysicalSize<u32>) -> Vec<UIVertex> {
        let mut cursor_vertices: Vec<UIVertex> = Vec::new();
        //竖下来的指针
        //顺时针四个点 从左上角开始
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_SIZE,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_LENGTH,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        //横着的指针
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 - CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 + CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices.push(UIVertex {
            position: [
                physical_size.width as f32 / 2.0 - CURSOR_HALF_LENGTH,
                physical_size.height as f32 / 2.0 + CURSOR_HALF_SIZE,
            ],
            color: [1.0, 1.0, 1.0, 0.7],
        });
        cursor_vertices
    }

    pub fn draw_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        left: f32,
        top: f32,
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.ui_text_renderer
            .draw_text(device, queue, left, top, encoder, view);
    }

    pub fn resize(&mut self, queue: &Queue, physical_size: winit::dpi::PhysicalSize<u32>) {
        let ui_uniform = UIUniform {
            screen_size: [physical_size.width as f32, physical_size.height as f32],
        };

        self.cursor_vertices = Self::generate_cursor_vertices(physical_size);
        queue.write_buffer(
            &self.cursor_vertex_buffer,
            0,
            bytemuck::cast_slice(&self.cursor_vertices),
        );

        queue.write_buffer(
            &self.screen_size_uniform_buffer,
            0,
            bytemuck::cast_slice(&[ui_uniform]),
        );
    }

    pub fn draw_ui(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        diffuse_bind_group: &BindGroup,
        block_materials_bind_group: &BindGroup,
        left: f32,
        top: f32,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        //self.ui_text_renderer
        //    .text_renderer
        //    .prepare(
        //        device,
        //        queue,
        //        &mut self.ui_text_renderer.font_system,
        //        &mut self.ui_text_renderer.atlas,
        //        &self.ui_text_renderer.view_port,
        //        [TextArea {
        //            buffer: &self.ui_text_renderer.text_buffer,
        //            left,
        //            top,
        //            scale: 1.0,
        //            bounds: TextBounds {
        //                left: 0,
        //                top: 0,
        //                right: 600,
        //                bottom: 160,
        //            },
        //            default_color: Color::rgb(255, 0, 255),
        //            custom_glyphs: &[],
        //        }],
        //        &mut self.ui_text_renderer.swash_cache,
        //    )
        //    .unwrap();

        render_pass.set_pipeline(&self.inventory_renderer.render_pipeline);
        render_pass.set_bind_group(0, &self.screen_size_uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.cursor_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.cursor_index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..CURSOR_INDEX.len() as _, 0, 0..1);

        render_pass.set_bind_group(0, &self.screen_size_uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.inventory_renderer.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.inventory_renderer.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.inventory_renderer.index_buffer.slice(..),
            IndexFormat::Uint16,
        );
        render_pass.draw_indexed(
            0..INVENTORY_INDEX.len() as _,
            0,
            0..self.inventory_renderer.instances.len() as _,
        );

        render_pass.set_pipeline(&self.block_renderer.render_pipeline);
        render_pass.set_bind_group(0, &self.screen_size_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, diffuse_bind_group, &[]);
        render_pass.set_bind_group(2, block_materials_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.block_renderer.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.block_renderer.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.block_renderer.index_buffer.slice(..),
            IndexFormat::Uint16,
        );
        render_pass.draw_indexed(
            0..realm::INDICES.len() as _,
            0,
            0..self.block_renderer.instances.len() as _,
        );

        //self.ui_text_renderer
        //    .text_renderer
        //    .render(
        //        &self.ui_text_renderer.atlas,
        //        &self.ui_text_renderer.view_port,
        //        &mut render_pass,
        //    )
        //    .unwrap();
    }
}
