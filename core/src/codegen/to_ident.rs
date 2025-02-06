use crate::font::Glyph;
use std::collections::HashMap;

/// Maps a set of glyphs to categories with identifiers
pub fn to_categories(glyphs: &[Glyph]) -> HashMap<String, HashMap<String, Glyph>> {
    let mut categories = HashMap::new();
    for glyph in glyphs {
        let (category, name) = glyph.name().to_category();
        let category = category.unwrap_or_else(|| "Other".to_string());

        let identifier = uniquify(&name, |id| {
            categories
                .get(&category)
                .map_or(true, |c: &HashMap<String, Glyph>| !c.contains_key(id))
        });

        let category = categories.entry(category).or_insert_with(HashMap::new);
        category.insert(identifier, glyph.clone());
    }

    categories
}

/// Maps a set of glyphs to identifiers, checking for duplicates
pub fn to_identifiers(glyphs: &[Glyph]) -> HashMap<String, Glyph> {
    let mut identifiers = HashMap::new();
    for glyph in glyphs {
        let mut identifier = glyph.name().to_identifier();

        // Check for dupes
        identifier = uniquify(&identifier, |id| !identifiers.contains_key(id));
        identifiers.insert(identifier, glyph.clone());
    }

    identifiers
}

/// Generates a unique identifier from an identifier
pub fn uniquify<F: Fn(&str) -> bool>(name: &str, is_unique: F) -> String {
    let mut identifier = name.to_string();
    if !is_unique(&identifier) {
        identifier.push_str("Alt");

        // Check for dupes again until we find a unique identifier
        if !is_unique(&identifier) {
            let mut idn = 2;
            let mut buffer = itoa::Buffer::new();
            loop {
                let idn_f = buffer.format(idn);
                let mut id = String::with_capacity(identifier.len() + idn_f.len());
                id.push_str(&identifier);
                id.push_str(idn_f);
                if is_unique(&id) {
                    identifier = id;
                    break;
                }

                idn += 1;
            }
        }
    }

    identifier
}

#[allow(dead_code)]
pub trait ToIdentExt {
    /// Converts a font string to a valid Rust identifier
    /// Font strings use . - _ and alphanumeric characters
    fn to_identifier(&self) -> String;

    /// Converts a font string to a valid Rust module name
    fn to_modname(&self) -> String;

    /// Returns the prefix and the rest of the font string
    fn to_category(&self) -> (Option<String>, String);

    /// Merges two identifiers into a single identifier
    fn merge_identifiers(&self, other: &str) -> String;
}
impl ToIdentExt for str {
    fn to_category(&self) -> (Option<String>, String) {
        let parts = self.splitn(2, '-').collect::<Vec<_>>();
        match parts.as_slice() {
            [prefix, rest] => (Some(prefix.to_identifier()), rest.to_identifier()),
            [rest] => (None, rest.to_identifier()),
            _ => (None, "_".to_string()),
        }
    }

    fn to_modname(&self) -> String {
        let s = self.replace(['.', '-'], "_").to_lowercase();
        if RUST_KEYWORDS.binary_search(&s.as_str()).is_ok() {
            format!("_{s}")
        } else {
            s
        }
    }

    fn to_identifier(&self) -> String {
        //
        // Replace all occurrences of . and - with _
        let mut identifier = self.replace(['.', '-'], "_");

        //
        // Replace all _[a-z] pairs with the uppercase letter
        let mut chars = identifier.chars();
        let mut new_identifier = String::with_capacity(identifier.len());
        while let Some(c) = chars.next() {
            if c == '_' {
                if let Some(next) = chars.next() {
                    new_identifier.push(next.to_ascii_uppercase());
                } else {
                    new_identifier.push(c);
                }
            } else {
                new_identifier.push(c);
            }
        }
        identifier = new_identifier;

        //
        // If the identifier starts with a digit, prepend an underscore
        match identifier.chars().next() {
            // If the string is empty, `_` is a valid identifier
            None => return "_".to_string(),

            Some(c) if c.is_ascii_digit() => {
                // Prepend an underscore for digits
                identifier.insert(0, '_');
            }

            Some(c) => {
                // Set first character to uppercase
                let first = c.to_string().to_uppercase();
                identifier = first + &identifier[1..];
            }
        }

        //
        // If the identifier is a reserved keyword, prepend an underscore
        if identifier.as_str() == "Self" {
            identifier.insert(0, '_');
        }

        identifier
    }

    fn merge_identifiers(&self, other: &str) -> String {
        let other = other.strip_prefix('_').unwrap_or(other);
        format!("{self}{other}")
    }
}

const RUST_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];
