use cosmic_text::Wrap;
use tiny_skia::{BYTES_PER_PIXEL, PixmapMut};

use crate::{
    config::{Config, WkEntry},
    layer::{
        color::{OPAQUE_ALPHA, WkColorPixelOps},
        text::WkText,
        unit::{Offset, Size},
    },
};

pub(crate) struct WkRender<'a> {
    pub(crate) text: WkText<'a>,
    config: &'a Config,
    entries: Vec<&'a WkEntry>,
}

impl<'a> WkRender<'a> {
    pub(crate) fn new(config: &'a Config, entries: Vec<&'a WkEntry>, text: WkText<'a>) -> Self {
        Self {
            text,
            config,
            entries,
        }
    }
}

impl<'r> WkRender<'r> {
    pub(crate) fn draw(&mut self, size: Size<u32>, canvas: &mut [u8]) {
        let entries = &self.entries;

        let mut pixmap = PixmapMut::from_bytes(canvas, size.width(), size.height())
            .expect("Can't create PixmapMut");
        pixmap.fill(self.config.color.bg.into());
        let pixmap_data = pixmap.data_mut();

        let stride = size.width() as usize * BYTES_PER_PIXEL;
        let max_height = pixmap_data.len() / stride;
        let usable_w = self.config.without_padding(size.width());
        let mut current_y = self.config.layout.padding;

        let pref_w = self
            .text
            .max_width(entries.iter().map(|e| &e.prefix).collect());
        let sepr_w = self
            .text
            .max_width(entries.iter().map(|e| &e.separator).collect());
        let desc_w = usable_w - pref_w - sepr_w;

        let fg: cosmic_text::Color = self.config.color.fg.into();

        for entry in entries {
            let usable_h = (size.height() - current_y - self.config.layout.padding).min(max_height as u32);

            self.text.set_size(Size::new(pref_w, size.height()).into());
            self.text.set_wrap(Wrap::None);
            self.text.set_text(&entry.prefix);
            Self::inner_draw(
                &mut self.text,
                pixmap_data,
                Offset::new(self.config.layout.padding, current_y),
                Size::new(pref_w, usable_h),
                stride,
                fg,
            );

            self.text.set_size(Size::new(sepr_w, size.height()).into());
            self.text.set_wrap(Wrap::None);
            self.text.set_text(&entry.separator);
            Self::inner_draw(
                &mut self.text,
                pixmap_data,
                Offset::new(self.config.layout.padding + pref_w, current_y),
                Size::new(sepr_w, usable_h),
                stride,
                fg,
            );

            self.text.set_size(Size::new(desc_w, size.height()).into());
            self.text.set_wrap(Wrap::Word);
            self.text.set_text(&entry.description);
            Self::inner_draw(
                &mut self.text,
                pixmap_data,
                Offset::new(self.config.layout.padding + pref_w + sepr_w, current_y),
                Size::new(desc_w, usable_h),
                stride,
                fg,
            );

            let lines_offset = self.text.lines_h(&entry.description, desc_w);

            current_y += lines_offset;
        }
    }

    pub(crate) fn inner_draw(
        text: &mut WkText,
        pixmap_data: &mut [u8],
        offset: Offset<u32>,
        usable: Size<u32>,
        stride: usize,
        fg: cosmic_text::Color,
    ) {
        let bound_x = offset.x() as i32..((offset.x() + usable.width()) as i32);
        let bound_y = offset.y() as i32..((offset.y() + usable.height()) as i32);

        text.render_glyph(
            Offset::new(offset.x(), offset.y()).into(),
            fg,
            |physical, color| {
                if bound_x.contains(&physical.x()) && bound_y.contains(&physical.y()) {
                    let idx = (physical.y() as usize * stride)
                        + (physical.x() as usize * BYTES_PER_PIXEL);
                    // ARGB8888 → [B, G, R, A]
                    let alpha = color.a().normalize_alpha();

                    let (b, g, r, a) = (idx, idx + 1, idx + 2, idx + 3);
                    pixmap_data[b] = color.b().blend_to(pixmap_data[b], alpha);
                    pixmap_data[g] = color.g().blend_to(pixmap_data[g], alpha);
                    pixmap_data[r] = color.r().blend_to(pixmap_data[r], alpha);
                    pixmap_data[a] = OPAQUE_ALPHA.blend_to(pixmap_data[a], alpha);
                }
            },
        );
    }
}
