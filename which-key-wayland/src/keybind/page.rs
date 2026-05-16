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
