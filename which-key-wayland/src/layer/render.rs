use cosmic_text::Wrap;
use tiny_skia::{BYTES_PER_PIXEL, PixmapMut};

use crate::{
    config::{Config, Footer, SYMBOL_INDICATOR},
    keybind::page::Page,
    layer::{
        color::{OPAQUE_ALPHA, WkColorPixelOps},
        text::WkText,
        unit::{Offset, Size},
    },
};

pub struct WkRender;

impl WkRender {
    pub fn draw(
        config: &Config,
        wk_text: &mut WkText,
        size: Size<u32>,
        canvas: &mut [u8],
        entries: &Page,
        header: Option<(&str, &str)>,
    ) {
        let mut pixmap = PixmapMut::from_bytes(canvas, size.width(), size.height())
            .expect("Can't create PixmapMut");
        pixmap.fill(config.color.bg.into());
        let pixmap_data = pixmap.data_mut();

        let stride = size.width() as usize * BYTES_PER_PIXEL;
        let usable_w = config.without_padding(size.width());
        let mut current_y = config.layout.padding;

        let key_strings: Vec<String> = entries.items.iter().map(|(k, _)| k.to_string()).collect();
        let key_w = wk_text.max_width(key_strings.iter().map(|s| s.as_str()).collect());
        let padded_indicator = config.font.size.floor() as u32;
        let ind_w = wk_text.max_width(vec![SYMBOL_INDICATOR]) + padded_indicator + padded_indicator;
        let des_w = usable_w - key_w - ind_w;

        // Header
        {
            match header {
                None => {
                    wk_text.set_size(Size::new(usable_w, size.height()).into());
                    wk_text.set_wrap(Wrap::None);
                    wk_text.set_text("Which-Key");
                    Self::inner_draw(
                        wk_text,
                        pixmap_data,
                        Offset::new(config.layout.padding, current_y),
                        Size::new(usable_w, size.height() - current_y),
                        stride,
                        config.color.fg_key.into(),
                    );
                    current_y += config.font.line_height as u32;
                }
                Some((_, group_desc)) => {
                    wk_text.set_size(Size::new(usable_w, size.height()).into());
                    wk_text.set_wrap(Wrap::Word);
                    wk_text.set_text(group_desc);
                    Self::inner_draw(
                        wk_text,
                        pixmap_data,
                        Offset::new(config.layout.padding, current_y),
                        Size::new(usable_w, size.height() - current_y),
                        stride,
                        config.color.fg_description.into(),
                    );
                    let lines_offset = wk_text.lines_h(group_desc, usable_w);
                    current_y += lines_offset;
                }
            }
        }

        // Separate between header and entries
        {
            current_y += config.font.line_height.floor() as u32;
        }

        // Entries
        {
            let max_height = pixmap_data.len() / stride;

            for entry in &entries.items {
                let usable_h =
                    (size.height() - current_y - config.layout.padding).min(max_height as u32);
                let (key, bind) = entry;

                {
                    wk_text.set_size(Size::new(key_w, size.height()).into());
                    wk_text.set_wrap(Wrap::None);
                    wk_text.set_text(&key.to_string());
                    Self::inner_draw(
                        wk_text,
                        pixmap_data,
                        Offset::new(config.layout.padding, current_y),
                        Size::new(key_w, usable_h),
                        stride,
                        config.color.fg_key.into(),
                    );
                }

                {
                    wk_text.set_size(Size::new(ind_w, size.height()).into());
                    wk_text.set_wrap(Wrap::None);
                    wk_text.set_text(SYMBOL_INDICATOR);
                    Self::inner_draw(
                        wk_text,
                        pixmap_data,
                        Offset::new(config.layout.padding + key_w + padded_indicator, current_y),
                        Size::new(ind_w, usable_h),
                        stride,
                        config.color.fg_separator.into(),
                    );
                }

                {
                    wk_text.set_size(Size::new(des_w, size.height()).into());
                    wk_text.set_wrap(Wrap::Word);
                    wk_text.set_text(&bind.desc);
                    Self::inner_draw(
                        wk_text,
                        pixmap_data,
                        Offset::new(config.layout.padding + key_w + ind_w, current_y),
                        Size::new(des_w, usable_h),
                        stride,
                        config.color.fg_description.into(),
                    );
                }

                let lines_offset = wk_text.lines_h(&bind.desc, des_w);
                current_y += lines_offset;
            }
        }

        // Footer
        {
            current_y += config.font.line_height.floor() as u32;

            let usable_h = (size.height() - current_y - config.layout.padding).min({
                let max_height = pixmap_data.len() / stride;
                max_height as u32
            });

            let key_color: cosmic_text::Color = config.color.fg_key.into();
            let desc_color: cosmic_text::Color = config.color.fg_description.into();

            let footer = Footer::default();
            let spans: Vec<_> = footer
                .items
                .iter()
                .enumerate()
                .flat_map(|(i, (k, d))| {
                    [
                        (i > 0).then_some(("  ", desc_color)),
                        Some((k.as_str(), key_color)),
                        Some((" ", desc_color)),
                        Some((d.as_str(), desc_color)),
                    ]
                    .into_iter()
                    .flatten()
                })
                .collect();

            wk_text.set_size(Size::new(usable_w, size.height()).into());
            wk_text.set_wrap(Wrap::Word);
            wk_text.set_rich_text(spans.as_slice());
            Self::inner_draw(
                wk_text,
                pixmap_data,
                Offset::new(config.layout.padding, current_y),
                Size::new(usable_w, usable_h),
                stride,
                desc_color,
            );
        }
    }

    fn inner_draw(
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

                    // ABGR8888 → [R, G, B, A]
                    let alpha = color.a().normalize_alpha();
                    let (r, g, b, a) = (idx, idx + 1, idx + 2, idx + 3);
                    pixmap_data[r] = color.r().blend_to(pixmap_data[r], alpha);
                    pixmap_data[g] = color.g().blend_to(pixmap_data[g], alpha);
                    pixmap_data[b] = color.b().blend_to(pixmap_data[b], alpha);
                    pixmap_data[a] = OPAQUE_ALPHA.blend_to(pixmap_data[a], alpha);
                }
            },
        );
    }
}
