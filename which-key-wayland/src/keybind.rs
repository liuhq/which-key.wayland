pub(crate) mod actions;
pub(crate) mod page;

use std::{collections::BTreeMap, rc::Rc};

use crate::keybind::actions::Action;

#[derive(Debug)]
pub struct Bind {
    pub bind: BindKind,
    pub separator: Rc<str>,
    pub desc: String,
}

#[derive(Debug, Default)]
pub struct KeyBindMap {
    pub(crate) map: BTreeMap<String, Bind>,
}

impl KeyBindMap {
    pub fn new(map: BTreeMap<String, Bind>) -> Self {
        Self { map }
    }
}

pub fn normalize_key_string(key: &str) -> String {
    let mut parts: Vec<&str> = key.split('+').collect();
    let base = parts.pop().unwrap_or("");
    parts.sort();
    parts.push(base);
    parts.join("+")
}

#[derive(Debug)]
pub enum BindKind {
    Action(Vec<Action>),
    Group(KeyBindMap),
}
