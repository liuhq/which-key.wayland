use smithay_client_toolkit::{
    delegate_keyboard,
    reexports::client::{
        Connection, QueueHandle,
        protocol::{wl_keyboard, wl_surface},
    },
    seat::keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
    shell::WaylandSurface,
};

use std::str::FromStr;
use std::time::Instant;

use crate::{
    keybind::{BindKind, key::Key, page::PageDirection},
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
        if let Some(ref layer) = self.layer
            && layer.wl_surface() == surface
        {
            log::debug!("Keyboard focus on surface with pressed syms: {keysyms:?}");
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
        if let Some(ref layer) = self.layer
            && layer.wl_surface() == surface
        {
            log::debug!("Release keyboard focus on surface");
            self.hide_overlay()
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
        log::debug!("Key press: {event:?}");

        self.last_key_time = Some(Instant::now());

        if event.keysym == Keysym::Escape {
            if self.key_path.pop().is_some() {
                self.next_cursor = None;
                self.prev_cursor = None;
                self.draw(None, PageDirection::Forward);
            } else {
                self.hide_overlay();
            }
            return;
        }

        if self.modifiers.ctrl {
            match event.keysym {
                Keysym::d => {
                    if let Some(idx) = self.next_cursor.take() {
                        self.draw(Some(idx), PageDirection::Forward);
                    }
                    return;
                }
                Keysym::u => {
                    if let Some(idx) = self.prev_cursor.take() {
                        self.draw(Some(idx), PageDirection::Backward);
                    }
                    return;
                }
                _ => {}
            }
        }

        let Some(key_str) = keysym_to_key_string(event.keysym, &self.modifiers) else {
            return;
        };

        let Ok(key) = Key::from_str(&key_str) else {
            return;
        };

        let action = {
            let map = self.current_bind_map();
            map.map.get(&key).map(|b| match &b.bind {
                BindKind::Action(actions) => Some(actions.clone()),
                BindKind::Group(_) => None,
            })
        };

        match action {
            Some(Some(actions)) => {
                for action in &actions {
                    if let Err(e) = action.run() {
                        log::error!("Action error: {e}");
                    }
                }
                self.hide_overlay();
            }
            Some(None) => {
                self.key_path.push(key);
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
        log::trace!("Key repeat: {event:?}");
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        log::trace!("Key release: {event:?}");
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

#[cfg(test)]
mod tests {
    use super::*;
    use smithay_client_toolkit::seat::keyboard::Modifiers;
    use xkbcommon::xkb;

    fn ks(name: &str) -> Keysym {
        xkb::keysym_from_name(name, xkb::KEYSYM_NO_FLAGS)
    }

    fn mods(ctrl: bool, shift: bool, alt: bool, logo: bool) -> Modifiers {
        Modifiers {
            ctrl,
            shift,
            alt,
            logo,
            ..Modifiers::default()
        }
    }

    #[test]
    fn alpha_key_no_modifiers() {
        let result = keysym_to_key_string(ks("a"), &mods(false, false, false, false));
        assert_eq!(result, Some("A".to_string()));
    }

    #[test]
    fn alpha_key_uppercase() {
        let result = keysym_to_key_string(ks("A"), &mods(false, false, false, false));
        assert_eq!(result, Some("A".to_string()));
    }

    #[test]
    fn alpha_key_with_ctrl() {
        let result = keysym_to_key_string(ks("c"), &mods(true, false, false, false));
        assert_eq!(result, Some("Ctrl+C".to_string()));
    }

    #[test]
    fn alpha_key_with_shift() {
        let result = keysym_to_key_string(ks("x"), &mods(false, true, false, false));
        assert_eq!(result, Some("Shift+X".to_string()));
    }

    #[test]
    fn alpha_key_with_alt() {
        let result = keysym_to_key_string(ks("m"), &mods(false, false, true, false));
        assert_eq!(result, Some("Alt+M".to_string()));
    }

    #[test]
    fn alpha_key_with_super() {
        let result = keysym_to_key_string(ks("r"), &mods(false, false, false, true));
        assert_eq!(result, Some("Super+R".to_string()));
    }

    #[test]
    fn alpha_key_with_all_modifiers_sorted() {
        let result = keysym_to_key_string(ks("q"), &mods(true, true, true, true));
        assert_eq!(result, Some("Alt+Ctrl+Shift+Super+Q".to_string()));
    }

    #[test]
    fn non_alpha_keysym_returns_none() {
        assert!(keysym_to_key_string(ks("Escape"), &mods(false, false, false, false)).is_none());
        assert!(keysym_to_key_string(ks("Return"), &mods(false, false, false, false)).is_none());
        assert!(keysym_to_key_string(ks("F1"), &mods(false, false, false, false)).is_none());
        assert!(keysym_to_key_string(ks("1"), &mods(false, false, false, false)).is_none());
        assert!(keysym_to_key_string(ks("Tab"), &mods(false, false, false, false)).is_none());
    }

    #[test]
    fn non_alpha_with_modifiers_still_returns_none() {
        assert!(keysym_to_key_string(ks("Escape"), &mods(true, false, false, false)).is_none());
    }
}
