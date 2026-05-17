use std::ops::Bound;

use crate::keybind::{Bind, KeyBindMap, key::Key};

#[derive(Debug)]
pub struct Page<'a> {
    pub items: Vec<(&'a Key, &'a Bind)>,
    pub next_cursor: Option<&'a Key>,
    pub prev_cursor: Option<&'a Key>,
}

#[derive(Debug, Clone, Copy)]
pub enum PageDirection {
    Forward,
    Backward,
}

impl KeyBindMap {
    pub fn page(
        &self,
        cursor: Option<&Key>,
        direction: PageDirection,
        page_size: usize,
    ) -> Page<'_> {
        let items = match direction {
            PageDirection::Forward => self.collect_forward(cursor, page_size),
            PageDirection::Backward => self.collect_backward(cursor, page_size),
        };

        let next_cursor = self.probe_next(items.last());
        let prev_cursor = self.probe_prev(items.first());

        Page {
            items,
            next_cursor,
            prev_cursor,
        }
    }

    fn collect_forward(&self, after: Option<&Key>, page_size: usize) -> Vec<(&Key, &Bind)> {
        let range = match after {
            Some(key) => self
                .map
                .range::<Key, _>((Bound::Excluded(key), Bound::Unbounded)),
            None => self.map.range::<Key, _>(..),
        };
        range.take(page_size).collect()
    }

    fn collect_backward(&self, before: Option<&Key>, page_size: usize) -> Vec<(&Key, &Bind)> {
        let range = match before {
            Some(key) => self
                .map
                .range::<Key, _>((Bound::Unbounded, Bound::Excluded(key))),
            None => self.map.range::<Key, _>(..),
        };
        let mut items: Vec<_> = range.rev().take(page_size).collect();
        items.reverse();
        items
    }

    fn probe_next<'a>(&self, last: Option<&(&'a Key, &'a Bind)>) -> Option<&'a Key> {
        let (key, _) = last?;
        self.map
            .range::<Key, _>((Bound::Excluded(*key), Bound::Unbounded))
            .next()
            .map(|_| *key)
    }

    fn probe_prev<'a>(&self, first: Option<&(&'a Key, &'a Bind)>) -> Option<&'a Key> {
        let (key, _) = first?;
        self.map
            .range::<Key, _>((Bound::Unbounded, Bound::Excluded(*key)))
            .next_back()
            .map(|_| *key)
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
        let cursor = mk_key("B");
        let page = map.page(Some(&cursor), PageDirection::Forward, 3);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].0, &mk_key("C"));
        assert_eq!(page.items[1].0, &mk_key("D"));
        assert_eq!(page.items[2].0, &mk_key("E"));
    }

    #[test]
    fn page_forward_from_last() {
        let map = mk_map(&["A", "B"]);
        let cursor = mk_key("B");
        let page = map.page(Some(&cursor), PageDirection::Forward, 3);
        assert!(page.items.is_empty());
    }

    #[test]
    fn page_backward_from_cursor() {
        let map = mk_map(&["A", "B", "C", "D", "E"]);
        let cursor = mk_key("D");
        let page = map.page(Some(&cursor), PageDirection::Backward, 3);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].0, &mk_key("A"));
        assert_eq!(page.items[1].0, &mk_key("B"));
        assert_eq!(page.items[2].0, &mk_key("C"));
    }

    #[test]
    fn page_backward_from_first() {
        let map = mk_map(&["A", "B"]);
        let cursor = mk_key("A");
        let page = map.page(Some(&cursor), PageDirection::Backward, 3);
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
    fn page_forward_with_next_cursor() {
        let map = mk_map(&["A", "B", "C", "D", "E"]);
        let page = map.page(None, PageDirection::Forward, 3);
        assert!(page.next_cursor.is_some());
        assert_eq!(page.next_cursor.unwrap(), &mk_key("C"));
    }

    #[test]
    fn page_forward_no_next_cursor_at_end() {
        let map = mk_map(&["A", "B"]);
        let page = map.page(None, PageDirection::Forward, 5);
        assert!(page.next_cursor.is_none());
    }

    #[test]
    fn page_forward_prev_cursor_is_none_from_start() {
        let map = mk_map(&["A", "B", "C"]);
        let page = map.page(None, PageDirection::Forward, 2);
        assert!(page.prev_cursor.is_none());
    }

    #[test]
    fn page_forward_prev_cursor_from_middle() {
        let map = mk_map(&["A", "B", "C", "D"]);
        let cursor = mk_key("B");
        let page = map.page(Some(&cursor), PageDirection::Forward, 2);
        assert!(page.prev_cursor.is_some());
        assert_eq!(page.prev_cursor.unwrap(), &mk_key("C"));
    }

    #[test]
    fn empty_map_returns_empty_page() {
        let map = KeyBindMap::default();
        let page = map.page(None, PageDirection::Forward, 10);
        assert!(page.items.is_empty());
        assert!(page.next_cursor.is_none());
        assert!(page.prev_cursor.is_none());
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
}
