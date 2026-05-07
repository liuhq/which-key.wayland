use std::rc::Rc;

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{
        Connection, EventQueue,
        globals::registry_queue_init,
        protocol::{wl_keyboard, wl_shm},
    },
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{KeyboardInteractivity, Layer, LayerShell, LayerSurface},
    },
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
    pub config: Rc<Config>,
    pub wk_text: WkText,
}

impl WkLayer {
    pub fn new(config: Config) -> (Self, EventQueue<Self>) {
        let mut wk_text = WkText::new(config.font.size, config.font.line_height);
        let init_height = WkLayer::final_height(&config, &mut wk_text);

        let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
        let (globals, event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr_layer_shell not available");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

        let surface = compositor.create_surface(&qh);
        let layer = layer_shell.create_layer_surface(
            &qh,
            surface,
            Layer::Overlay,
            Some("simple_layer"),
            None,
        );

        layer.set_anchor(config.layout.anchor);
        layer.set_margin(
            config.layout.margin.top,
            config.layout.margin.right,
            config.layout.margin.bottom,
            config.layout.margin.left,
        );
        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        layer.set_size(config.layout.width, init_height);
        layer.commit();

        let pool = SlotPool::new((config.layout.width * 4 * init_height) as usize, &shm)
            .expect("Failed to create pool");

        (
            Self {
                registry_state: RegistryState::new(&globals),
                output_state: OutputState::new(&globals, &qh),
                seat_state: SeatState::new(&globals, &qh),
                shm,
                pool,
                buffer: None,
                layer,
                keyboard: None,

                exit: false,
                first_configure: true,
                keyboard_focus: false,
                config: Rc::new(config),
                wk_text,
            },
            event_queue,
        )
    }
}

impl WkLayer {
    pub fn run(&mut self, event_queue: &mut EventQueue<Self>) {
        loop {
            event_queue.blocking_dispatch(self).unwrap();
            if self.exit {
                log::info!("Exiting wk_layer");
                break;
            }
        }
    }

    pub fn draw(&mut self) {
        let width = self.config.layout.width;
        let height = Self::final_height(&self.config, &mut self.wk_text);

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
            let entries = self.config.bind.page(
                None,
                PageDirection::Forward,
                self.config.layout.max_items as usize,
            );
            WkRender::draw(
                &self.config,
                &mut self.wk_text,
                Size::new(width, height),
                canvas,
                &entries,
            );
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

impl WkLayer {
    pub fn final_height(config: &Config, wk_text: &mut WkText) -> u32 {
        let mut total_lines = config.with_padding(0);
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
}
