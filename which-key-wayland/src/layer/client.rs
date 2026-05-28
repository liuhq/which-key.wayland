use std::os::fd::{AsFd, OwnedFd};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{
        Connection, EventQueue, QueueHandle,
        globals::registry_queue_init,
        protocol::{wl_keyboard, wl_seat, wl_shm},
    },
    registry::RegistryState,
    seat::{SeatState, keyboard::Modifiers},
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

use crate::config::reloader::ConfigReloader;
use crate::config::{Footer, SYMBOL_INDICATOR};
use crate::{
    config::Config,
    ipc,
    keybind::{BindKind, KeyBindMap, key::Key, page::PageDirection},
    layer::{render::WkRender, text::WkText, unit::Size},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Showing,
    Hidden,
    Exiting,
}

pub struct WhichKey {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub seat_state: SeatState,
    pub compositor: CompositorState,
    pub layer_shell: LayerShell,
    pub shm: Shm,
    pub pool: SlotPool,
    pub buffer: Option<SlotBuffer>,
    pub layer: Option<LayerSurface>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub wl_seat: Option<wl_seat::WlSeat>,

    pub state: AppState,
    pub first_configure: bool,
    pub timeout: Duration,
    pub config: Rc<Config>,
    pub config_reloader: Option<ConfigReloader>,
    pub wk_text: WkText,
    pub next_cursor: Option<usize>,
    pub prev_cursor: Option<usize>,
    pub modifiers: Modifiers,
    pub key_path: Vec<Key>,
    pub last_key_time: Option<Instant>,

    pub dbus_rx: mpsc::Receiver<ipc::DBusCommand>,
    pub wake_fd: OwnedFd,
}

impl WhichKey {
    pub fn new(
        config: Config,
        config_path: Option<PathBuf>,
        dbus_rx: mpsc::Receiver<ipc::DBusCommand>,
        wake_fd: OwnedFd,
    ) -> (Self, EventQueue<Self>) {
        let mut wk_text = WkText::new(config.font.size, config.font.line_height);
        let init_height =
            WhichKey::calc_h(&config, &mut wk_text, None, PageDirection::Forward, &[]);

        let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
        let (globals, event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr_layer_shell not available");
        let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

        let surface = compositor.create_surface(&qh);
        let layer =
            layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("wk_layer"), None);

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
                compositor,
                layer_shell,
                shm,
                pool,
                buffer: None,
                layer: Some(layer),
                keyboard: None,
                wl_seat: None,

                state: AppState::Showing,
                first_configure: true,
                timeout: Duration::from_millis(config.timeout as u64),
                config: Rc::new(config),
                config_reloader: config_path.map(ConfigReloader::init_mtime),
                wk_text,
                next_cursor: None,
                prev_cursor: None,
                modifiers: Modifiers::default(),
                key_path: Vec::new(),
                last_key_time: None,

                dbus_rx,
                wake_fd,
            },
            event_queue,
        )
    }
}

impl WhichKey {
    pub fn hide_overlay(&mut self) {
        log::info!("Hiding overlay");
        self.key_path.clear();
        self.next_cursor = None;
        self.prev_cursor = None;
        self.last_key_time = None;
        self.buffer = None;
        self.layer = None;
        if let Some(kbd) = self.keyboard.take() {
            kbd.release();
        }
        self.state = AppState::Hidden;
    }

    pub fn show_overlay(&mut self, qh: &QueueHandle<Self>) {
        if self.layer.is_some() {
            self.hide_overlay();
        }

        if let Some(cr @ ConfigReloader::Mtime { .. }) = &mut self.config_reloader
            && cr.has_changed_by_mtime()
        {
            log::debug!("check mtime -> config changed");
            self.reload_config();
        }

        let height = Self::calc_h(
            &self.config,
            &mut self.wk_text,
            None,
            PageDirection::Forward,
            &self.key_path,
        );

        let surface = self.compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Overlay,
            Some("wk_layer"),
            None,
        );

        layer.set_anchor(self.config.layout.anchor);
        layer.set_margin(
            self.config.layout.margin.top,
            self.config.layout.margin.right,
            self.config.layout.margin.bottom,
            self.config.layout.margin.left,
        );
        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        layer.set_size(self.config.layout.width, height);
        layer.commit();

        self.layer = Some(layer);
        self.first_configure = true;
        self.state = AppState::Showing;

        if self.keyboard.is_none()
            && let Some(ref seat) = self.wl_seat
        {
            match self.seat_state.get_keyboard(qh, seat, None) {
                Ok(kbd) => self.keyboard = Some(kbd),
                Err(e) => log::error!("Failed to create keyboard: {e}"),
            }
        }

        log::info!("Showing overlay");
    }

    pub fn run(&mut self, event_queue: &mut EventQueue<Self>) {
        loop {
            event_queue.dispatch_pending(self).unwrap();

            while let Ok(cmd) = self.dbus_rx.try_recv() {
                match cmd {
                    ipc::DBusCommand::Show => {
                        log::debug!("DBus::Show");
                        if matches!(self.state, AppState::Hidden) {
                            self.show_overlay(&event_queue.handle());
                        } else {
                            self.hide_overlay();
                            self.show_overlay(&event_queue.handle());
                        }
                    }
                    ipc::DBusCommand::Quit => {
                        log::debug!("DBus::Quit");
                        self.state = AppState::Exiting;
                    }
                    ipc::DBusCommand::Reload => {
                        log::debug!("DBus::Reload");
                        if let Some(cr @ ConfigReloader::Mtime { .. }) = &mut self.config_reloader {
                            let mtime = cr.try_read_mtime();
                            cr.sync_mtime(mtime);
                        }
                        self.reload_config();
                    }
                }
            }

            match self.state {
                AppState::Exiting => {
                    log::info!("Exiting WhichKey");
                    break;
                }
                AppState::Showing => {
                    if self.timeout > Duration::ZERO
                        && let Some(last) = self.last_key_time
                        && last.elapsed() >= self.timeout
                    {
                        self.hide_overlay();
                    }
                }
                AppState::Hidden => {}
            }

            let poll_dur = match (self.state, self.last_key_time) {
                (AppState::Showing, Some(last)) if self.timeout > Duration::ZERO => {
                    self.timeout.checked_sub(last.elapsed())
                }
                _ => None,
            };

            event_queue.flush().unwrap();
            if let Some(guard) = event_queue.prepare_read() {
                let wayland_fd = guard.connection_fd();
                let wake_borrowed = self.wake_fd.as_fd();

                let dbus_woken;
                {
                    let mut fds = [
                        rustix::event::PollFd::new(
                            &wayland_fd,
                            rustix::event::PollFlags::IN | rustix::event::PollFlags::ERR,
                        ),
                        rustix::event::PollFd::new(&wake_borrowed, rustix::event::PollFlags::IN),
                    ];
                    let ts = poll_dur.and_then(|d| rustix::event::Timespec::try_from(d).ok());
                    let _ = rustix::event::poll(&mut fds, ts.as_ref());
                    dbus_woken = fds[1].revents().contains(rustix::event::PollFlags::IN);
                }

                let _ = guard.read();

                if dbus_woken {
                    let mut buf = [0u8; 8];
                    let _ = rustix::io::read(&self.wake_fd, &mut buf);
                }
            }
        }
    }

    fn reload_config(&mut self) {
        let Some(cr) = &mut self.config_reloader else {
            return;
        };
        match cr {
            ConfigReloader::Mtime { path, .. } => {
                let new_config = Config::load(path);
                self.timeout = Duration::from_millis(new_config.timeout as u64);
                self.config = Rc::new(new_config);
            }
            ConfigReloader::Inotify { .. } => todo!(),
        }
    }

    pub fn current_bind_map(&self) -> &KeyBindMap {
        let mut map = &self.config.bind;
        for key in &self.key_path {
            if let Some(bind) = map.map.get(key)
                && let BindKind::Group(group) = &bind.bind
            {
                map = group;
                continue;
            }
            break;
        }
        map
    }

    fn parent_bind_map(&self) -> &KeyBindMap {
        let mut map = &self.config.bind;
        let len = self.key_path.len();
        if len > 0 {
            for key in &self.key_path[..len - 1] {
                if let Some(bind) = map.map.get(key)
                    && let BindKind::Group(group) = &bind.bind
                {
                    map = group;
                    continue;
                }
                break;
            }
        }
        map
    }

    pub fn draw(&mut self, cursor: Option<usize>, direction: PageDirection) {
        let Some(ref layer) = self.layer else {
            return;
        };

        let width = self.config.layout.width;
        let height = Self::calc_h(
            &self.config,
            &mut self.wk_text,
            cursor,
            direction,
            &self.key_path,
        );

        let config = Rc::clone(&self.config);
        let max_items = self.config.layout.max_items as usize;
        let page = {
            let mut map = &config.bind;
            for key in &self.key_path {
                if let Some(bind) = map.map.get(key)
                    && let BindKind::Group(group) = &bind.bind
                {
                    map = group;
                    continue;
                }
                break;
            }
            let total = map.len();
            let page = map.page(cursor, direction, max_items);
            let page_start = match cursor {
                Some(c) => match direction {
                    PageDirection::Forward => (c + 1).min(total),
                    PageDirection::Backward => c.saturating_sub(max_items),
                },
                None => match direction {
                    PageDirection::Forward => 0,
                    PageDirection::Backward => total.saturating_sub(max_items),
                },
            };
            self.next_cursor = (!page.items.is_empty() && page_start + page.items.len() < total)
                .then(|| page_start + page.items.len() - 1);
            self.prev_cursor = (page_start > 0).then_some(page_start);
            page
        };

        let header = self.key_path.last().and_then(|last_key| {
            self.parent_bind_map()
                .map
                .get(last_key)
                .map(|bind| (last_key.to_string(), bind.desc.clone()))
        });
        let header_ref: Option<(&str, &str)> =
            header.as_ref().map(|(k, d)| (k.as_str(), d.as_str()));

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                width as i32 * 4,
                wl_shm::Format::Abgr8888,
            )
            .expect("Failed to create buffer");

        layer.set_size(width, height);

        WkRender::draw(
            &self.config,
            &mut self.wk_text,
            Size::new(width, height),
            canvas,
            &page,
            header_ref,
        );

        layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);

        layer.commit();

        self.buffer = Some(buffer);
    }
}

impl WhichKey {
    pub fn calc_h(
        config: &Config,
        wk_text: &mut WkText,
        cursor: Option<usize>,
        direction: PageDirection,
        key_path: &[Key],
    ) -> u32 {
        let mut map = &config.bind;
        let mut last_key_desc = None;
        for key in key_path {
            if let Some(bind) = map.map.get(key)
                && let BindKind::Group(group) = &bind.bind
            {
                map = group;
                last_key_desc = Some(&bind.desc);
                continue;
            }
            break;
        }

        let separator = config.font.line_height.ceil() as u32;
        let usable_w = config.without_padding(config.layout.width);
        let header = last_key_desc.map_or(0, |d| wk_text.lines_h(d, usable_w));
        let mut total_lines =
            config.with_padding(0) + header + if header == 0 { 0 } else { separator };

        let entries = map.page(cursor, direction, config.layout.max_items as usize);
        let key_strings: Vec<String> = entries.items.iter().map(|(k, _)| k.to_string()).collect();
        let key_w = wk_text.max_width(key_strings.iter().map(|s| s.as_str()).collect());
        let padded_indicator = config.font.size.floor() as u32;
        let ind_w = wk_text.max_width(vec![SYMBOL_INDICATOR]) + padded_indicator + padded_indicator;
        let des_w = usable_w - key_w - ind_w;
        for (_, bind) in entries.items.iter() {
            let des_h = wk_text.lines_h(&bind.desc, des_w);
            total_lines += des_h;
        }

        let footer = wk_text.lines_h(&Footer::default().to_string(), usable_w);
        total_lines += separator + footer;

        total_lines
    }
}
