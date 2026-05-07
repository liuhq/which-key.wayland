pub mod actions;
pub mod page;

use std::{collections::BTreeMap, rc::Rc};

use crate::keybind::actions::Action;

#[derive(Debug)]
pub struct Bind {
    pub bind: BindKind,
    pub separator: Rc<str>,
    pub desc: String,
}

#[derive(Debug)]
pub struct KeyBindMap {
    map: BTreeMap<String, Bind>,
}

impl KeyBindMap {
    pub fn new(map: BTreeMap<String, Bind>) -> Self {
        Self { map }
    }
}

#[derive(Debug)]
pub enum BindKind {
    Action(Vec<Action>),
    Group(KeyBindMap),
}
