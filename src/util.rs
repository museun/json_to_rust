use crate::CasingScheme;
use std::collections::HashSet;

pub const KEYWORDS: &[&str] = &[
    "abstract", "alignof", "as", "async", "await", "become", "box", "break", "const", "continue",
    "crate", "do", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "offsetof", "override", "priv", "proc",
    "pub", "pure", "ref", "return", "self", "Self", "sizeof", "static", "struct", "super", "trait",
    "true", "try", "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while",
    "yield",
];

pub const BUILTIN: &[&str] = &[
    "bool", "Box", "f64", "i64", "Option", "Result", "String", "Vec",
];

pub fn fix_name(name: &str, used: &mut HashSet<String>, casing: CasingScheme) -> String {
    // TODO ascii-fy everything until rust allows utf-8 identifiers
    let name = name.trim();
    let mut out = match name.chars().next() {
        Some('0'..='9') => casing.convert(&format!("n{}", name)),
        _ => casing.convert(name),
    };

    if KEYWORDS.contains(&&*out) {
        out.push('_');
    }

    if BUILTIN.contains(&&*out) {
        out.push('_')
    }

    assert!(!out.is_empty());

    let mut i = 1;
    // clone so we have the original string to try a new suffix with
    let mut temp = out.clone();
    loop {
        if !used.contains(&temp) {
            // fast path. if we don't need to rename on the first try, don't
            // make a 2nd allocation
            if i == 1 {
                used.insert(temp);
                break out;
            }

            used.insert(temp.clone());
            break temp;
        }
        i += 1;
        temp = format!("{}{}", out, i);
    }
}

pub static TUPLE_WRAPPER: Wrapper = Wrapper {
    left: "(",
    right: ")",
};

pub static NOOP_WRAPPER: Wrapper = Wrapper::new();

#[derive(Copy, Clone, Debug)]
pub struct Wrapper {
    pub left: &'static str,
    pub right: &'static str,
}

impl Default for Wrapper {
    fn default() -> Self {
        NOOP_WRAPPER
    }
}

impl Wrapper {
    pub const fn new() -> Self {
        Self {
            left: "",
            right: "",
        }
    }

    pub fn from_string(left: String) -> Self {
        let left = Box::leak(left.into_boxed_str());
        Self { left, right: ">" }
    }

    pub fn apply(&self, item: String) -> String {
        if self.left.is_empty() && self.right.is_empty() {
            return item;
        }
        format!("{}{}{}", self.left, item, self.right)
    }
}
