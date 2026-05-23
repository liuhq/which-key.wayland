use crate::keybind::{Bind, BindKind, KeyBindMap, key::Key};

#[derive(Debug)]
pub struct Page<'a> {
    pub items: Vec<(&'a Key, &'a Bind)>,
}

#[derive(Debug, Clone, Copy)]
pub enum PageDirection {
    Forward,
    Backward,
}

impl KeyBindMap {
    fn ordered(&self) -> Vec<(&Key, &Bind)> {
        let (actions, groups): (Vec<_>, Vec<_>) = self
            .map
            .iter()
            .partition(|(_, b)| matches!(b.bind, BindKind::Action(_)));
        actions.into_iter().chain(groups).collect()
    }

    pub fn len(&self) -> usize {
        self.ordered().len()
    }

    pub fn page(
        &self,
        cursor: Option<usize>,
        direction: PageDirection,
        page_size: usize,
    ) -> Page<'_> {
        let ordered = self.ordered();
        let (start, end) = match cursor {
            Some(c) => match direction {
                PageDirection::Forward => {
                    let s = (c + 1).min(ordered.len());
                    (s, (s + page_size).min(ordered.len()))
                }
                PageDirection::Backward => {
                    let s = c.saturating_sub(page_size);
                    (s, c)
                }
            },
            None => match direction {
                PageDirection::Forward => (0, page_size.min(ordered.len())),
                PageDirection::Backward => {
                    let s = ordered.len().saturating_sub(page_size);
                    (s, ordered.len())
                }
            },
        };
        Page {
            items: ordered[start..end].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keybind::{Bind, BindKind};
    use std::collections::BTreeMap;

    fn mk_key(s: &str) -> Key {
        s.parse().unwrap()
    }

    fn mk_bind(desc: &str) -> Bind {
        Bind {
            bind: BindKind::Action(Vec::new()),
            desc: desc.to_string(),
        }
    }

    fn mk_group(desc: &str) -> Bind {
        Bind {
            bind: BindKind::Group(KeyBindMap::default()),
            desc: desc.to_string(),
        }
    }

    fn mk_map(keys: &[&str]) -> KeyBindMap {
        let mut map = BTreeMap::new();
        for (i, k) in keys.iter().enumerate() {
            map.insert(mk_key(k), mk_bind(&format!("desc{i}")));
        }
        KeyBindMap::new(map)
    }

    #[test]
    fn page_forward_from_start() {
        let map = mk_map(&["A", "B", "C", "D", "E"]);
        let page = map.page(None, PageDirection::Forward, 3);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].0, &mk_key("A"));
        assert_eq!(page.items[1].0, &mk_key("B"));
        assert_eq!(page.items[2].0, &mk_key("C"));
    }

    #[test]
    fn page_forward_from_cursor() {
        let map = mk_map(&["A", "B", "C", "D", "E"]);
        let page = map.page(Some(1), PageDirection::Forward, 3);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].0, &mk_key("C"));
        assert_eq!(page.items[1].0, &mk_key("D"));
        assert_eq!(page.items[2].0, &mk_key("E"));
    }

    #[test]
    fn page_forward_from_last() {
        let map = mk_map(&["A", "B"]);
        let page = map.page(Some(1), PageDirection::Forward, 3);
        assert!(page.items.is_empty());
    }

    #[test]
    fn page_backward_from_cursor() {
        let map = mk_map(&["A", "B", "C", "D", "E"]);
        let page = map.page(Some(3), PageDirection::Backward, 3);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].0, &mk_key("A"));
        assert_eq!(page.items[1].0, &mk_key("B"));
        assert_eq!(page.items[2].0, &mk_key("C"));
    }

    #[test]
    fn page_backward_from_first() {
        let map = mk_map(&["A", "B"]);
        let page = map.page(Some(0), PageDirection::Backward, 3);
        assert!(page.items.is_empty());
    }

    #[test]
    fn page_backward_from_start() {
        let map = mk_map(&["A", "B", "C", "D", "E"]);
        let page = map.page(None, PageDirection::Backward, 3);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].0, &mk_key("C"));
        assert_eq!(page.items[1].0, &mk_key("D"));
        assert_eq!(page.items[2].0, &mk_key("E"));
    }

    #[test]
    fn empty_map_returns_empty_page() {
        let map = KeyBindMap::default();
        let page = map.page(None, PageDirection::Forward, 10);
        assert!(page.items.is_empty());
    }

    #[test]
    fn page_size_larger_than_map() {
        let map = mk_map(&["A", "B"]);
        let page = map.page(None, PageDirection::Forward, 10);
        assert_eq!(page.items.len(), 2);
    }

    #[test]
    fn page_size_zero() {
        let map = mk_map(&["A", "B", "C"]);
        let page = map.page(None, PageDirection::Forward, 0);
        assert!(page.items.is_empty());
    }

    #[test]
    fn mixed_action_group_ordering() {
        let mut map = BTreeMap::new();
        map.insert(mk_key("A"), mk_group("group A"));
        map.insert(mk_key("B"), mk_bind("action B"));
        map.insert(mk_key("C"), mk_group("group C"));
        map.insert(mk_key("D"), mk_bind("action D"));
        let map = KeyBindMap::new(map);

        let page = map.page(None, PageDirection::Forward, 10);
        let keys: Vec<&Key> = page.items.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![&mk_key("B"), &mk_key("D"), &mk_key("A"), &mk_key("C")]);
    }

    #[test]
    fn len_returns_total_ordered_count() {
        let mut map = BTreeMap::new();
        map.insert(mk_key("A"), mk_group("group A"));
        map.insert(mk_key("B"), mk_bind("action B"));
        let map = KeyBindMap::new(map);

        assert_eq!(map.len(), 2);
    }
}
