use glyphon::*;
use wgpu::MultisampleState;
use winit::dpi::PhysicalSize;

pub struct UITextRenderer {
    pub view_port: Viewport,
    pub font_system: FontSystem,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_buffer: Buffer,
    pub swash_cache: SwashCache,
}

impl UITextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        scale_factor: f32,
        physical_size: PhysicalSize<u32>,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
        let mut text_buffer = Buffer::new(
            &mut font_system,
            Metrics {
                font_size: 30.0,
                line_height: 42.0,
            },
        );

        let physical_width = physical_size.width as f32 * scale_factor;
        let physical_height = physical_size.height as f32 * scale_factor;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.set_text(
            &mut font_system,
            "Hello world👋!你好世界！🦁",
            Attrs::new(),
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        let view_port = Viewport::new(&device, &cache);

        let swash_cache = SwashCache::new();

        UITextRenderer {
            font_system,
            atlas,
            text_renderer,
            text_buffer,
            view_port,
            swash_cache,
        }
    }

    pub fn draw_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        left: f32,
        top: f32,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.view_port,
                [TextArea {
                    buffer: &self.text_buffer,
                    left,
                    top,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();

        {
            let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // 使用Load保留前面渲染的内容
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // 不使用深度测试
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.text_renderer
                .render(&self.atlas, &self.view_port, &mut ui_render_pass)
                .unwrap();
        } // UI渲染通道结束
    }

    pub fn set_text(&mut self, text: &str) {
        self.text_buffer
            .set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
        self.text_buffer
            .shape_until_scroll(&mut self.font_system, false);
    }
}
