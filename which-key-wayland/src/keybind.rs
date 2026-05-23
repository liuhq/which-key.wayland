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
