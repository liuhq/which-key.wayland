use smithay_client_toolkit::{
    delegate_keyboard,
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_keyboard, wl_surface},
    },
    seat::keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
    shell::WaylandSurface,
};

use std::time::Instant;

use crate::{
    keybind::{BindKind, page::PageDirection},
    layer::client::WhichKey,
};

fn keysym_to_key_string(keysym: Keysym, modifiers: &Modifiers) -> Option<String> {
    let name = xkbcommon::xkb::keysym_get_name(keysym);

    if name.len() == 1 && name.as_bytes()[0].is_ascii_alphabetic() {
        let base = name.to_ascii_uppercase();

        let mut parts = Vec::new();
        if modifiers.ctrl {
            parts.push("Ctrl");
        }
        if modifiers.alt {
            parts.push("Alt");
        }
        if modifiers.logo {
            parts.push("Super");
        }
        if modifiers.shift {
            parts.push("Shift");
        }
        parts.sort();
        parts.push(&base);
        Some(parts.join("+"))
    } else {
        None
    }
}

impl KeyboardHandler for WhichKey {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        keysyms: &[Keysym],
    ) {
        if self.layer.wl_surface() == surface {
            println!("Keyboard focus on window with pressed syms: {keysyms:?}");
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            println!("Release keyboard focus on window");
            self.keyboard_focus = false;

            self.exit = true
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key press: {event:?}");

        self.last_key_time = Some(Instant::now());

        if event.keysym == Keysym::Escape {
            if self.key_path.pop().is_some() {
                self.next_cursor = None;
                self.prev_cursor = None;
                self.draw(None, PageDirection::Forward);
            } else {
                self.exit = true;
            }
            return;
        }

        if self.modifiers.ctrl {
            match event.keysym {
                Keysym::d => {
                    if let Some(nc) = self.next_cursor.take() {
                        self.draw(Some(&nc), PageDirection::Forward);
                    }
                    return;
                }
                Keysym::u => {
                    if let Some(pc) = self.prev_cursor.take() {
                        self.draw(Some(&pc), PageDirection::Backward);
                    }
                    return;
                }
                _ => {}
            }
        }

        let Some(key_str) = keysym_to_key_string(event.keysym, &self.modifiers) else {
            return;
        };

        let keysym_str = key_str;
        let action = {
            let map = self.current_bind_map();
            map.map.get(&keysym_str).map(|b| match &b.bind {
                BindKind::Action(actions) => Some(actions.clone()),
                BindKind::Group(_) => None,
            })
        };

        match action {
            Some(Some(actions)) => {
                for action in &actions {
                    if let Err(e) = action.run() {
                        eprintln!("Action error: {e}");
                    }
                }
                self.exit = true;
            }
            Some(None) => {
                self.key_path.push(keysym_str);
                self.next_cursor = None;
                self.prev_cursor = None;
                self.draw(None, PageDirection::Forward);
            }
            None => {}
        }
    }

    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        println!("Key repeat: {event:?}");
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key release: {event:?}");
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        _layout: u32,
    ) {
        self.modifiers = modifiers;
    }
}

delegate_keyboard!(WhichKey);
