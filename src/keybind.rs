pub mod actions;

use std::collections::HashMap;

use crate::keybind::actions::Action;

#[derive(Debug)]
pub struct Bind {
    pub bind: BindKind,
    pub desc: String,
}

pub type KeyBindMap = HashMap<String, Bind>;

#[derive(Debug)]
pub enum BindKind {
    Action(Vec<Action>),
    Group(KeyBindMap),
}
