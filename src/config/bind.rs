use std::collections::HashMap;

use kdl::{KdlDocument, KdlNode, KdlValue};

use crate::keybind::{Bind, BindKind, KeyBindMap, actions::Action};

pub fn bind_parser(config: &KdlDocument) -> anyhow::Result<KeyBindMap> {
    let Some(binds) = config
        .get("bind")
        .and_then(|n| n.children())
        .map(|d| d.nodes())
    else {
        anyhow::bail!("bind: not found or empty")
    };

    let map = parse_binds_(binds)?;

    Ok(map)
}

fn parse_binds_<'a, I: IntoIterator<Item = &'a KdlNode>>(nodes: I) -> anyhow::Result<KeyBindMap> {
    let nodes: Vec<_> = nodes.into_iter().collect();
    let mut map = HashMap::with_capacity(nodes.len());

    for node in nodes {
        let desc = match node.get("desc") {
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
            BindKind::Group(parse_binds_(groups)?)
        };

        map.insert(node.name().value().to_string(), Bind { bind, desc });
    }

    Ok(map)
}
