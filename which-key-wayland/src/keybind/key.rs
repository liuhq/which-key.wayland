use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modifier {
    Super,
    Shift,
    Ctrl,
    Alt,
}

impl Modifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Modifier::Super => "Super",
            Modifier::Shift => "Shift",
            Modifier::Ctrl => "Ctrl",
            Modifier::Alt => "Alt",
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub modifiers: Option<Vec<Modifier>>,
    pub base: String,
}

impl Key {
    pub fn new(modifiers: Option<Vec<Modifier>>, base: String) -> Self {
        let modifiers = modifiers.map(|mut m| {
            m.sort();
            m
        });

        Self {
            modifiers,
            base: Self::title_case(&base),
        }
    }

    fn title_case(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str().to_lowercase().as_str(),
        }
    }
}

impl FromStr for Key {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split('+').collect();

        if parts.is_empty() || parts.iter().all(|p| p.is_empty()) {
            anyhow::bail!("empty key string");
        }

        let base = parts.last().unwrap();
        if base.is_empty() {
            anyhow::bail!("empty base key");
        }

        let mod_parts = &parts[..parts.len() - 1];
        if mod_parts.is_empty() {
            return Ok(Key::new(None, base.to_string()));
        }

        let mut modifiers = Vec::new();
        for m in mod_parts {
            if m.is_empty() {
                anyhow::bail!("empty modifier");
            }
            let modifier = match *m {
                "Super" | "super" => Modifier::Super,
                "Shift" | "shift" => Modifier::Shift,
                "Ctrl" | "ctrl" => Modifier::Ctrl,
                "Alt" | "alt" => Modifier::Alt,
                _ => anyhow::bail!("invalid modifier: {m}"),
            };
            modifiers.push(modifier);
        }

        Ok(Key::new(Some(modifiers), base.to_string()))
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        if let Some(mods) = &self.modifiers {
            let mut sorted = mods.clone();
            sorted.sort();

            for m in sorted {
                if !first {
                    f.write_str("+")?;
                }
                first = false;
                f.write_str(m.as_str())?;
            }
        }

        if !first {
            f.write_str("+")?;
        }

        f.write_str(&self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_modifiers() {
        let key: Key = "a".parse().unwrap();
        assert_eq!(key, Key::new(None, "A".to_string()));
    }

    #[test]
    fn base_with_uppercase() {
        let key: Key = "A".parse().unwrap();
        assert_eq!(key, Key::new(None, "A".to_string()));
    }

    #[test]
    fn base_as_word() {
        let key: Key = "Delete".parse().unwrap();
        assert_eq!(key, Key::new(None, "Delete".to_string()));
    }

    #[test]
    fn lowercase_base_gets_title_cased() {
        let key: Key = "ctrl+alt+delete".parse().unwrap();
        assert_eq!(
            key,
            Key::new(
                Some(vec![Modifier::Ctrl, Modifier::Alt]),
                "Delete".to_string()
            )
        );
    }

    #[test]
    fn mixed_case_base_gets_title_cased() {
        let key: Key = "eScApE".parse().unwrap();
        assert_eq!(key, Key::new(None, "Escape".to_string()));
    }

    #[test]
    fn numeric_base_preserved() {
        let key: Key = "F12".parse().unwrap();
        assert_eq!(key, Key::new(None, "F12".to_string()));
    }

    #[test]
    fn single_modifier() {
        let key: Key = "ctrl+c".parse().unwrap();
        assert_eq!(key, Key::new(Some(vec![Modifier::Ctrl]), "C".to_string()));
    }

    #[test]
    fn multiple_modifiers() {
        let key: Key = "Super+Shift+A".parse().unwrap();
        assert_eq!(
            key,
            Key::new(
                Some(vec![Modifier::Super, Modifier::Shift]),
                "A".to_string()
            )
        );
    }

    #[test]
    fn modifiers_are_sorted_by_new() {
        let key: Key = "Shift+Super+A".parse().unwrap();
        assert_eq!(
            key,
            Key::new(
                Some(vec![Modifier::Shift, Modifier::Super,]),
                "A".to_string()
            )
        );
        assert_eq!(key.modifiers, Some(vec![Modifier::Super, Modifier::Shift,]));
    }

    #[test]
    fn round_trip_via_display() {
        let original: Key = "Ctrl+Alt+Delete".parse().unwrap();
        let displayed = original.to_string();
        let reparsed: Key = displayed.parse().unwrap();
        assert_eq!(original, reparsed);
    }

    #[test]
    fn round_trip_single_modifier() {
        let original: Key = "Super+Escape".parse().unwrap();
        let displayed = original.to_string();
        let reparsed: Key = displayed.parse().unwrap();
        assert_eq!(original, reparsed);
    }

    #[test]
    fn round_trip_no_modifiers() {
        let original: Key = "F1".parse().unwrap();
        let displayed = original.to_string();
        let reparsed: Key = displayed.parse().unwrap();
        assert_eq!(original, reparsed);
    }

    #[test]
    fn error_empty_string() {
        let result: Result<Key, _> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn error_trailing_plus() {
        let result: Result<Key, _> = "Super+".parse();
        assert!(result.is_err());
    }

    #[test]
    fn error_leading_plus() {
        let result: Result<Key, _> = "+A".parse();
        assert!(result.is_err());
    }

    #[test]
    fn error_consecutive_plus() {
        let result: Result<Key, _> = "Super++A".parse();
        assert!(result.is_err());
    }

    #[test]
    fn error_invalid_modifier() {
        let result: Result<Key, _> = "Foo+Bar+Baz".parse();
        assert!(result.is_err());
    }

    #[test]
    fn error_partial_invalid_modifier() {
        let result: Result<Key, _> = "Super+Foo+A".parse();
        assert!(result.is_err());
    }

    #[test]
    fn all_modifiers() {
        let key: Key = "Super+Shift+Ctrl+Alt+X".parse().unwrap();
        assert_eq!(
            key,
            Key::new(
                Some(vec![
                    Modifier::Super,
                    Modifier::Shift,
                    Modifier::Ctrl,
                    Modifier::Alt
                ]),
                "X".to_string()
            )
        );
    }
}
