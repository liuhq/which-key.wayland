pub mod actions;
pub mod key;
pub mod page;

use std::collections::BTreeMap;

use crate::keybind::{actions::Action, key::Key};

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
