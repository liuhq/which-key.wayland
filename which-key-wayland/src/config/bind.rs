use std::{collections::BTreeMap, str::FromStr};

use kdl::{KdlDocument, KdlNode, KdlValue};

use crate::{
    config::define::SYMBOL_GROUP,
    keybind::{Bind, BindKind, KeyBindMap, actions::Action, key::Key},
};

pub fn bind_parser(config: &KdlDocument) -> anyhow::Result<KeyBindMap> {
    let Some(binds) = config
        .get("bind")
        .and_then(|n| n.children())
        .map(|d| d.nodes())
    else {
        return Ok(KeyBindMap::default());
    };

    let map = parse_binds_(binds)?;

    Ok(map)
}

fn parse_binds_<'a, I: IntoIterator<Item = &'a KdlNode>>(nodes: I) -> anyhow::Result<KeyBindMap> {
    let nodes: Vec<_> = nodes.into_iter().collect();
    let mut map = BTreeMap::new();

    for node in nodes {
        let mut desc = match node.get("desc") {
            Some(KdlValue::String(desc)) => desc.to_string(),
            _ => String::new(),
        };

        let children: Vec<_> = node.iter_children().collect();
        let (actions, groups): (Vec<_>, Vec<_>) = children
            .iter()
            .partition::<Vec<&KdlNode>, _>(|n| Action::is_action(n));

        let bind = if groups.is_empty() {
            let actions = actions
                .into_iter()
                .map(Action::parse)
                .collect::<anyhow::Result<_>>()?;
            BindKind::Action(actions)
        } else {
            desc.insert_str(0, SYMBOL_GROUP);
            BindKind::Group(parse_binds_(groups)?)
        };

        map.insert(Key::from_str(node.name().value())?, Bind { bind, desc });
    }

    Ok(KeyBindMap::new(map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keybind::key::Key;

    fn parse(raw: &str) -> anyhow::Result<KeyBindMap> {
        let doc: KdlDocument = raw.parse()?;
        bind_parser(&doc)
    }

    #[test]
    fn empty_bind_block() {
        let raw = "bind {\n}";
        let map = parse(raw).unwrap();
        assert!(map.map.is_empty());
    }

    #[test]
    fn single_bind_spawn() {
        let raw = "bind {\n  B desc=\"Open browser\" { spawn \"google-chrome\"; }\n}";
        let map = parse(raw).unwrap();
        assert_eq!(map.map.len(), 1);
        let key: Key = "B".parse().unwrap();
        let bind = map.map.get(&key).unwrap();
        assert_eq!(bind.desc, "Open browser");
    }

    #[test]
    fn single_bind_sh() {
        let raw = "bind {\n  E desc=\"Test e\" { sh \"echo e\"; }\n}";
        let map = parse(raw).unwrap();
        assert_eq!(map.map.len(), 1);
        let key: Key = "E".parse().unwrap();
        let bind = map.map.get(&key).unwrap();
        assert_eq!(bind.desc, "Test e");
    }

    #[test]
    fn bind_without_desc() {
        let raw = "bind {\n  X { spawn \"app\"; }\n}";
        let map = parse(raw).unwrap();
        let key: Key = "X".parse().unwrap();
        let bind = map.map.get(&key).unwrap();
        assert_eq!(bind.desc, "");
    }

    #[test]
    fn bind_with_modifier_key() {
        let raw = "bind {\n  Ctrl+w desc=\"Close\" { sh \"echo w\"; }\n}";
        let map = parse(raw).unwrap();
        let key: Key = "Ctrl+w".parse().unwrap();
        let bind = map.map.get(&key).unwrap();
        assert_eq!(bind.desc, "Close");
    }

    #[test]
    fn nested_group_bind() {
        let raw =
            "bind {\n  A desc=\"Apps\" {\n    T desc=\"Terminal\" { spawn \"foot\"; }\n  }\n}";
        let map = parse(raw).unwrap();
        assert_eq!(map.map.len(), 1);
        let key_a: Key = "A".parse().unwrap();
        let bind_a = map.map.get(&key_a).unwrap();
        assert!(bind_a.desc.starts_with(SYMBOL_GROUP));
        assert!(bind_a.desc.ends_with("Apps"));
    }

    #[test]
    fn bind_invalid_key_returns_error() {
        let raw = "bind {\n  Foo+Bar desc=\"Bad\" { sh \"echo bad\"; }\n}";
        let result = parse(raw);
        assert!(result.is_err());
    }

    #[test]
    fn multiple_binds() {
        let raw = "bind {\n  A desc=\"Alpha\" { sh \"echo a\"; }\n  B desc=\"Beta\" { sh \"echo b\"; }\n}";
        let map = parse(raw).unwrap();
        assert_eq!(map.map.len(), 2);
    }

    #[test]
    fn bind_with_no_children_returns_empty_map() {
        let raw = "";
        let map = parse(raw).unwrap();
        assert!(map.map.is_empty());
    }

    #[test]
    fn bind_no_bind_node() {
        let raw = "other {\n  A desc=\"test\" { sh \"echo a\"; }\n}";
        let map = parse(raw).unwrap();
        assert!(map.map.is_empty());
    }

    #[test]
    fn group_prepends_symbol() {
        let raw =
            "bind {\n  G desc=\"Group name\" {\n    C desc=\"Child\" { spawn \"app\"; }\n  }\n}";
        let map = parse(raw).unwrap();
        let key: Key = "G".parse().unwrap();
        let bind = map.map.get(&key).unwrap();
        assert!(bind.desc.contains("Group name"));
        assert!(bind.desc.starts_with(SYMBOL_GROUP));
    }
}
