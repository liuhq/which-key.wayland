use std::collections::BTreeMap;

use kdl::{KdlDocument, KdlNode, KdlValue};

use super::define::ConfigSeparator;
use crate::keybind::{Bind, BindKind, KeyBindMap, actions::Action, normalize_key_string};

pub fn bind_parser(
    config: &KdlDocument,
    separator: &ConfigSeparator,
) -> anyhow::Result<KeyBindMap> {
    let Some(binds) = config
        .get("bind")
        .and_then(|n| n.children())
        .map(|d| d.nodes())
    else {
        return Ok(KeyBindMap::default());
    };

    let map = parse_binds_(binds, separator)?;

    Ok(map)
}

fn parse_binds_<'a, I: IntoIterator<Item = &'a KdlNode>>(
    nodes: I,
    separator: &ConfigSeparator,
) -> anyhow::Result<KeyBindMap> {
    let nodes: Vec<_> = nodes.into_iter().collect();
    let mut map = BTreeMap::new();

    for node in nodes {
        let desc = match node.get("desc") {
            Some(KdlValue::String(desc)) => desc.to_string(),
            _ => String::new(),
        };

        let children: Vec<_> = node.iter_children().collect();
        let (actions, groups): (Vec<_>, Vec<_>) = children
            .iter()
            .partition::<Vec<&KdlNode>, _>(|n| Action::is_action(n));

        let (bind, separator) = if groups.is_empty() {
            let actions = actions
                .into_iter()
                .map(Action::parse)
                .collect::<anyhow::Result<_>>()?;
            (BindKind::Action(actions), separator.action.clone())
        } else {
            (
                BindKind::Group(parse_binds_(groups, separator)?),
                separator.group.clone(),
            )
        };

        map.insert(
            normalize_key_string(node.name().value()),
            Bind {
                bind,
                separator,
                desc,
            },
        );
    }

    Ok(KeyBindMap::new(map))
}
