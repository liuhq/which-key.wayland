use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap};

use crate::layer::unit::{Offset, Size};

pub(crate) struct WkText<'a> {
    pub(crate) font_system: FontSystem,
    pub(crate) swash_cache: SwashCache,
    pub(crate) buffer: Buffer,
    metrics: Metrics,
    attrs: Attrs<'a>,
}

impl<'a> WkText<'a> {
    pub(crate) fn new(font_size: f32, line_height: f32) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let metrics = Metrics::new(font_size, line_height);
        let buffer = Buffer::new(&mut font_system, metrics);
        let attrs = Attrs::new().family(Family::Monospace);

        Self {
            font_system,
            swash_cache,
            buffer,
            attrs,
            metrics,
        }
    }

    pub(crate) fn set_size(&mut self, size: Size<f32>) {
        self.buffer.set_size(
            &mut self.font_system,
            Some(size.width()),
            Some(size.height()),
        );
    }

    pub(crate) fn set_wrap(&mut self, wrap: Wrap) {
        self.buffer.set_wrap(&mut self.font_system, wrap);
    }

    pub(crate) fn set_text(&mut self, text: &str) {
        let mut buffer = self.buffer.borrow_with(&mut self.font_system);
        buffer.set_text(text, &self.attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(false);
    }
}

impl<'a> WkText<'a> {
    pub(crate) fn max_width(&mut self, text: Vec<&str>) -> u32 {
        let mut buffer = self.buffer.borrow_with(&mut self.font_system);
        let mut max_w: f32 = 0.0;
        {
            buffer.set_size(Some(f32::MAX), Some(self.metrics.line_height));
            buffer.set_wrap(Wrap::None);

            for t in text {
                buffer.set_text(t, &self.attrs, Shaping::Advanced, None);
                buffer.shape_until_scroll(false);
                max_w = buffer
                    .layout_runs()
                    .map(|r| r.line_w.round())
                    .fold(max_w, f32::max);
            }
        }
        max_w.ceil() as u32
    }

    pub(crate) fn lines_h(&mut self, text: &str, width: u32) -> u32 {
        let mut buffer = self.buffer.borrow_with(&mut self.font_system);

        buffer.set_size(Some(width as f32), Some(f32::MAX));
        buffer.set_wrap(Wrap::Word);
        buffer.set_text(text, &self.attrs, Shaping::Advanced, None);

        buffer
            .layout_runs()
            .last()
            .map(|r| r.line_top + r.line_height)
            .unwrap_or(0.0)
            .round() as u32
    }

    pub(crate) fn render_glyph<F: FnMut(Offset<i32>, cosmic_text::Color)>(
        &mut self,
        offset: Offset<f32>,
        fg: cosmic_text::Color,
        mut f: F,
    ) {
        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((offset.x(), offset.y()), 1.0);

                self.swash_cache.with_pixels(
                    &mut self.font_system,
                    physical.cache_key,
                    fg,
                    |x, y, color| {
                        let px = physical.x + x;
                        let py = physical.y + y + run.line_y.round() as i32;

                        if color.a() == 0 {
                            return;
                        }

                        f(Offset::new(px, py), color)
                    },
                );
            }
        }
    }
}
