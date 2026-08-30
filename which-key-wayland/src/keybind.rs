pub mod actions;
pub mod key;
pub mod page;

use std::collections::BTreeMap;

use crate::{
    config::ConfigColor,
    keybind::{actions::Action, key::Key},
};

#[derive(Debug)]
pub struct Bind {
    pub bind: BindKind,
    pub desc: String,
}

#[derive(Debug, Default)]
pub struct KeyBindMap {
    pub map: BTreeMap<Key, Bind>,
}

impl KeyBindMap {
    pub fn new(map: BTreeMap<Key, Bind>) -> Self {
        Self { map }
    }
}

#[derive(Debug)]
pub enum BindKind {
    Action(Vec<Action>),
    Group(KeyBindMap),
}

impl BindKind {
    pub fn fg_from(&self, color: &ConfigColor) -> cosmic_text::Color {
        match self {
            BindKind::Action(_) => color.fg_action.into(),
            BindKind::Group(_) => color.fg_group.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_uses_action_foreground_color() {
        let colors = ConfigColor::default();
        let bind = BindKind::Action(Vec::new());

        assert_eq!(bind.fg_from(&colors), colors.fg_action.into());
    }

    #[test]
    fn group_uses_group_foreground_color() {
        let colors = ConfigColor::default();
        let bind = BindKind::Group(KeyBindMap::default());

        assert_eq!(bind.fg_from(&colors), colors.fg_group.into());
    }
}
