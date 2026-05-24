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
    if name.is_empty() {
        return None;
    }

    let utf8 = xkbcommon::xkb::keysym_to_utf8(keysym);
    let use_utf8 = utf8.chars().count() == 1
        && utf8.chars().next().is_some_and(|c| c.is_ascii_graphic());
    let base = if use_utf8 { utf8 } else { Key::title_case(&name) };

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
    if modifiers.shift && !use_utf8 {
        parts.push("Shift");
    }
    parts.sort();
    parts.push(&base);
    Some(parts.join("+"))
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
        assert_eq!(result, Some("a".to_string()));
    }

    #[test]
    fn alpha_key_uppercase() {
        let result = keysym_to_key_string(ks("A"), &mods(false, false, false, false));
        assert_eq!(result, Some("A".to_string()));
    }

    #[test]
    fn alpha_key_with_ctrl() {
        let result = keysym_to_key_string(ks("c"), &mods(true, false, false, false));
        assert_eq!(result, Some("Ctrl+c".to_string()));
    }

    #[test]
    fn alpha_key_with_shift() {
        let result = keysym_to_key_string(ks("x"), &mods(false, true, false, false));
        assert_eq!(result, Some("x".to_string()));
    }

    #[test]
    fn alpha_key_with_alt() {
        let result = keysym_to_key_string(ks("m"), &mods(false, false, true, false));
        assert_eq!(result, Some("Alt+m".to_string()));
    }

    #[test]
    fn alpha_key_with_super() {
        let result = keysym_to_key_string(ks("r"), &mods(false, false, false, true));
        assert_eq!(result, Some("Super+r".to_string()));
    }

    #[test]
    fn alpha_key_with_all_modifiers_sorted() {
        let result = keysym_to_key_string(ks("q"), &mods(true, true, true, true));
        assert_eq!(result, Some("Alt+Ctrl+Super+q".to_string()));
    }

    #[test]
    fn function_key() {
        let result = keysym_to_key_string(ks("F1"), &mods(false, false, false, false));
        assert_eq!(result, Some("F1".to_string()));
    }

    #[test]
    fn function_key_with_modifier() {
        let result = keysym_to_key_string(ks("F12"), &mods(true, false, false, false));
        assert_eq!(result, Some("Ctrl+F12".to_string()));
    }

    #[test]
    fn escape_key() {
        let result = keysym_to_key_string(ks("Escape"), &mods(false, false, false, false));
        assert_eq!(result, Some("Escape".to_string()));
    }

    #[test]
    fn escape_with_modifier() {
        let result = keysym_to_key_string(ks("Escape"), &mods(true, false, false, false));
        assert_eq!(result, Some("Ctrl+Escape".to_string()));
    }

    #[test]
    fn return_key() {
        let result = keysym_to_key_string(ks("Return"), &mods(false, false, false, false));
        assert_eq!(result, Some("Return".to_string()));
    }

    #[test]
    fn tab_key() {
        let result = keysym_to_key_string(ks("Tab"), &mods(false, false, false, false));
        assert_eq!(result, Some("Tab".to_string()));
    }

    #[test]
    fn space_key_title_cased() {
        let result = keysym_to_key_string(ks("space"), &mods(false, false, false, false));
        assert_eq!(result, Some("Space".to_string()));
    }

    #[test]
    fn digit_key() {
        let result = keysym_to_key_string(ks("1"), &mods(false, false, false, false));
        assert_eq!(result, Some("1".to_string()));
    }

    #[test]
    fn digit_key_with_modifier() {
        let result = keysym_to_key_string(ks("7"), &mods(true, false, false, true));
        assert_eq!(result, Some("Ctrl+Super+7".to_string()));
    }

    #[test]
    fn delete_key() {
        let result = keysym_to_key_string(ks("Delete"), &mods(false, false, false, false));
        assert_eq!(result, Some("Delete".to_string()));
    }

    #[test]
    fn backspace_key_title_cased() {
        let result = keysym_to_key_string(ks("BackSpace"), &mods(false, false, false, false));
        assert_eq!(result, Some("Backspace".to_string()));
    }

    #[test]
    fn arrow_key() {
        let result = keysym_to_key_string(ks("Up"), &mods(false, false, false, false));
        assert_eq!(result, Some("Up".to_string()));
    }

    #[test]
    fn home_key() {
        let result = keysym_to_key_string(ks("Home"), &mods(false, false, false, false));
        assert_eq!(result, Some("Home".to_string()));
    }

    #[test]
    fn semicolon_symbol() {
        let result = keysym_to_key_string(ks("semicolon"), &mods(false, false, false, false));
        assert_eq!(result, Some(";".to_string()));
    }

    #[test]
    fn semicolon_with_shift() {
        let result = keysym_to_key_string(ks("semicolon"), &mods(false, true, false, false));
        assert_eq!(result, Some(";".to_string()));
    }

    #[test]
    fn grave_backtick() {
        let result = keysym_to_key_string(ks("grave"), &mods(false, false, false, false));
        assert_eq!(result, Some("`".to_string()));
    }

    #[test]
    fn comma_symbol() {
        let result = keysym_to_key_string(ks("comma"), &mods(false, false, false, false));
        assert_eq!(result, Some(",".to_string()));
    }

    #[test]
    fn period_symbol() {
        let result = keysym_to_key_string(ks("period"), &mods(false, false, false, false));
        assert_eq!(result, Some(".".to_string()));
    }

    #[test]
    fn exclam_symbol() {
        let result = keysym_to_key_string(ks("exclam"), &mods(false, false, false, false));
        assert_eq!(result, Some("!".to_string()));
    }

    #[test]
    fn exclam_with_shift() {
        let result = keysym_to_key_string(ks("exclam"), &mods(false, true, false, false));
        assert_eq!(result, Some("!".to_string()));
    }

    #[test]
    fn space_unchanged() {
        let result = keysym_to_key_string(ks("space"), &mods(false, false, false, false));
        assert_eq!(result, Some("Space".to_string()));
    }

    #[test]
    fn shift_delete() {
        let result = keysym_to_key_string(ks("Delete"), &mods(false, true, false, false));
        assert_eq!(result, Some("Shift+Delete".to_string()));
    }

    #[test]
    fn shift_f1() {
        let result = keysym_to_key_string(ks("F1"), &mods(false, true, false, false));
        assert_eq!(result, Some("Shift+F1".to_string()));
    }

    #[test]
    fn shift_home() {
        let result = keysym_to_key_string(ks("Home"), &mods(false, true, false, false));
        assert_eq!(result, Some("Shift+Home".to_string()));
    }

    #[test]
    fn uppercase_keysym_with_shift() {
        let result = keysym_to_key_string(ks("A"), &mods(false, true, false, false));
        assert_eq!(result, Some("A".to_string()));
    }

    #[test]
    fn uppercase_keysym_with_ctrl() {
        let result = keysym_to_key_string(ks("C"), &mods(true, false, false, false));
        assert_eq!(result, Some("Ctrl+C".to_string()));
    }

    #[test]
    fn uppercase_keysym_with_ctrl_and_shift() {
        let result = keysym_to_key_string(ks("C"), &mods(true, true, false, false));
        assert_eq!(result, Some("Ctrl+C".to_string()));
    }
}
