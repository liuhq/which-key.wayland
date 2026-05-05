use std::ops::Bound;

use crate::keybind::{Bind, KeyBindMap};

pub struct Page<'a> {
    pub items: Vec<(&'a String, &'a Bind)>,
    pub next_cursor: Option<&'a str>,
    pub prev_cursor: Option<&'a str>,
}

pub enum PageDirection {
    Forward,
    Backward,
}

impl KeyBindMap {
    pub fn page(
        &self,
        cursor: Option<&str>,
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

    fn collect_forward(&self, after: Option<&str>, page_size: usize) -> Vec<(&String, &Bind)> {
        let range = match after {
            Some(key) => self
                .map
                .range::<str, _>((Bound::Excluded(key), Bound::Unbounded)),
            None => self.map.range::<str, _>(..),
        };
        range.take(page_size).collect()
    }

    fn collect_backward(&self, before: Option<&str>, page_size: usize) -> Vec<(&String, &Bind)> {
        let range = match before {
            Some(key) => self
                .map
                .range::<str, _>((Bound::Unbounded, Bound::Excluded(key))),
            None => self.map.range::<str, _>(..),
        };
        let mut items: Vec<_> = range.rev().take(page_size).collect();
        items.reverse();
        items
    }

    fn probe_next<'a>(&self, last: Option<&(&'a String, &'a Bind)>) -> Option<&'a str> {
        let (key, _) = last?;
        self.map
            .range::<str, _>((Bound::Excluded(key.as_str()), Bound::Unbounded))
            .next()
            .map(|_| key.as_str())
    }

    fn probe_prev<'a>(&self, first: Option<&(&'a String, &'a Bind)>) -> Option<&'a str> {
        let (key, _) = first?;
        self.map
            .range::<str, _>((Bound::Unbounded, Bound::Excluded(key.as_str())))
            .next_back()
            .map(|_| key.as_str())
    }
}
