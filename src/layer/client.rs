use smithay_client_toolkit::{
    output::OutputState,
    reexports::client::protocol::{wl_keyboard, wl_shm},
    registry::RegistryState,
    seat::SeatState,
    shell::{WaylandSurface, wlr_layer::LayerSurface},
    shm::{
        Shm,
        slot::{Buffer as SlotBuffer, SlotPool},
    },
};

pub mod compositor;
pub mod keyboard;
pub mod layer_shell;
pub mod output;
pub mod registry;
pub mod seat;
pub mod shm;

use crate::{
    config::Config,
    keybind::page::PageDirection,
    layer::{render::WkRender, text::WkText, unit::Size},
};

pub struct WkLayer {
    // wayland client
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub shm: Shm,
    pub pool: SlotPool,
    pub buffer: Option<SlotBuffer>,
    pub layer: LayerSurface,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,

    // state
    pub exit: bool,
    pub first_configure: bool,
    pub keyboard_focus: bool,
    pub config: Config,
    // pub wk_text: WkText,
}

impl WkLayer {
    pub(crate) fn final_height(config: &Config) -> u32 {
        let mut total_lines = config.with_padding(0);
        let mut wk_text = WkText::new(config.font.size, config.font.line_height);
        let entries = config.bind.page(
            None,
            PageDirection::Forward,
            config.layout.max_items as usize,
        );
        let key_w = wk_text.max_width(entries.items.iter().map(|(k, _)| k.as_str()).collect());
        let sep_w = wk_text.max_width(
            entries
                .items
                .iter()
                .map(|(_, b)| b.separator.as_ref())
                .collect(),
        );
        let des_w = config.without_padding(config.layout.width - key_w - sep_w);
        for (_, bind) in entries.items.iter() {
            let des_h = wk_text.lines_h(&bind.desc, des_w);
            total_lines += des_h;
        }
        total_lines
    }

    pub fn draw(&mut self) {
        let width = self.config.layout.width;
        let height = Self::final_height(&self.config);

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                width as i32 * 4,
                wl_shm::Format::Argb8888,
            )
            .expect("Failed to create buffer");

        self.layer.set_size(width, height);

        // Draw to the window:
        {
            let text = WkText::new(self.config.font.size, self.config.font.line_height);
            let entries = self.config.bind.page(
                None,
                PageDirection::Forward,
                self.config.layout.max_items as usize,
            );
            let mut render = WkRender::new(&self.config, entries, text);
            render.draw(Size::new(width, height), canvas);
        }

        // Damage the entire window
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Attach and commit to present.
        self.layer
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);

        self.layer.commit();

        self.buffer = Some(buffer);
    }
}
