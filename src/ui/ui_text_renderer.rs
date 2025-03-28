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
            "Hello world!你好世界！",
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
}
