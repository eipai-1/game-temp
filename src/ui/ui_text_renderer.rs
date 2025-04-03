use std::collections::BTreeMap;

use crate::{realm, ui::inventory_renderer};
use glyphon::*;
use wgpu::MultisampleState;
use winit::dpi::PhysicalSize;

pub const LINE_HEIGHT: f32 = 42.0;
pub const FONT_SIZE: f32 = 30.0;
pub const DEBUG_INFO_LEFT: f32 = 10.0;
pub const DEBUG_INFO_TOP: f32 = 10.0;
pub const DEBUG_INFO_COLOR: Color = Color::rgb(255, 255, 255);

pub struct TextEntry {
    pub buffer: Buffer,
    pub left: f32,
    pub top: f32,
    pub bounds: TextBounds,
    pub color: Color,
    pub id: String, // 用于标识文本，便于后续更新或移除
}

pub struct UITextRenderer {
    pub view_port: Viewport,
    pub font_system: FontSystem,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_entries: BTreeMap<String, TextEntry>,
    pub swash_cache: SwashCache,
    fps_display_interval: f64,
    fps_update_timer: f64,
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
                font_size: FONT_SIZE,
                line_height: LINE_HEIGHT,
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
            "",
            //"Hello world👋!你好世界！🦁",
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
            text_entries: BTreeMap::new(),
            view_port,
            swash_cache,
            fps_display_interval: 1.0,
            fps_update_timer: 0.0,
        }
    }

    // 绘制全部文本
    pub fn draw_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
        is_debug_info_open: bool,
    ) {
        // 如果没有文本，直接返回
        if self.text_entries.is_empty() {
            return;
        }

        // 构建TextArea数组，使用filter_map有选择地渲染
        let text_areas: Vec<TextArea> = self
            .text_entries
            .iter()
            .filter_map(|(id, entry)| {
                // 如果调试信息面板关闭，且文本是调试信息，则跳过
                if !is_debug_info_open
                    && (id == "player_position" || id == "fps_info" || id == "chunk_info")
                {
                    return None; // 不渲染这个文本
                }

                // 其他文本正常渲染
                Some(TextArea {
                    buffer: &entry.buffer,
                    left: entry.left,
                    top: entry.top,
                    scale: 1.0,
                    bounds: entry.bounds.clone(),
                    default_color: entry.color,
                    custom_glyphs: &[],
                })
            })
            .collect();

        // 如果过滤后没有文本需要渲染，则直接返回
        if text_areas.is_empty() {
            return;
        }

        // 调用prepare准备所有文本
        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.view_port,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        // 渲染所有文本
        self.text_renderer
            .render(&self.atlas, &self.view_port, render_pass)
            .unwrap();
    }

    // 添加新文本
    pub fn add_text(
        &mut self,
        id: &str,
        text: &str,
        left: f32,
        top: f32,
        bounds: TextBounds,
        color: Color,
    ) {
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics {
                font_size: 30.0,
                line_height: 42.0,
            },
        );

        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        // 将新文本插入到BTreeMap中
        self.text_entries.insert(
            id.to_string(),
            TextEntry {
                buffer,
                left,
                top,
                bounds,
                color,
                id: id.to_string(),
            },
        );
    }

    // 更新已有文本
    pub fn update_text(&mut self, id: &str, text: &str) -> bool {
        if let Some(entry) = self.text_entries.get_mut(id) {
            entry
                .buffer
                .set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
            entry
                .buffer
                .shape_until_scroll(&mut self.font_system, false);
            true
        } else {
            false
        }
    }

    // 移除文本
    pub fn remove_text(&mut self, id: &str) -> bool {
        self.text_entries.remove(id).is_some()
    }

    pub fn generate_debug_info(&mut self) {
        // 生成调试信息
        let debug_info = String::new();

        // 添加调试信息文本
        self.add_text(
            "player_position",
            &debug_info,
            DEBUG_INFO_LEFT,
            DEBUG_INFO_TOP + inventory_renderer::HOTBAR_TOP + inventory_renderer::SLOT_SIZE,
            TextBounds::default(),
            DEBUG_INFO_COLOR,
        );
        self.add_text(
            "fps_info",
            &debug_info,
            DEBUG_INFO_LEFT,
            DEBUG_INFO_TOP
                + inventory_renderer::HOTBAR_TOP
                + inventory_renderer::SLOT_SIZE
                + LINE_HEIGHT,
            TextBounds::default(),
            DEBUG_INFO_COLOR,
        );
        self.add_text(
            "chunk_info",
            &debug_info,
            DEBUG_INFO_LEFT,
            DEBUG_INFO_TOP
                + inventory_renderer::HOTBAR_TOP
                + inventory_renderer::SLOT_SIZE
                + LINE_HEIGHT * 2.0,
            TextBounds::default(),
            DEBUG_INFO_COLOR,
        );
    }

    pub fn update_debug_info(
        &mut self,
        position: cgmath::Point3<f32>,
        dt: f64,
        realm: &realm::Realm,
    ) {
        // 更新调试信息文本
        self.update_text(
            "player_position",
            &format!(
                "X:{:.4}, Y:{:.4}, Z:{:.4})",
                position.x, position.y, position.z
            ),
        );

        self.fps_update_timer += dt;
        if self.fps_update_timer >= self.fps_display_interval {
            self.fps_update_timer = 0.0;
            self.update_text("fps_info", format!("FPS: {:.0}", 1.0 / dt).as_str());
        }

        self.update_text(
            "chunk_info",
            &format!(
                "中心区块:({},{})  区块数量:{} pre_dx:{}, pre_dz:{} is_loading:{}",
                realm.data.center_chunk_pos.x,
                realm.data.center_chunk_pos.z,
                realm.data.chunk_map.len(),
                realm.pre_dx,
                realm.pre_dz,
                realm.is_loading,
            ),
        );
    }
}
