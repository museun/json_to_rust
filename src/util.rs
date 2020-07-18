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

trait WrapperApply {
    fn apply(&self, item: String) -> String;
}

#[derive(Clone, Debug)]
pub struct NestedWrapper {
    left: Wrapper,
    right: Wrapper,
}

#[derive(Clone, Debug)]
pub enum Wrapper {
    Bottom { left: String, right: String },
    Nested { left: Box<Self>, right: Box<Self> },
}

impl Wrapper {
    pub fn wrap(self, other: Self) -> Self {
        Self::Nested {
            left: Box::new(self),
            right: Box::new(other),
        }
    }

    pub fn new(left: impl Into<String>, right: impl Into<String>) -> Self {
        Self::Bottom {
            left: left.into(),
            right: right.into(),
        }
    }

    pub fn std_vec() -> Self {
        Self::custom_vec("Vec")
    }

    pub fn custom_vec(left: impl Into<String>) -> Self {
        let mut left = left.into();
        if !left.ends_with('<') {
            left.push('<')
        }
        Self::new(left, ">")
    }

    pub fn std_map() -> Self {
        Self::custom_map("HashMap<String, ")
    }

    pub fn custom_map(left: impl Into<String>) -> Self {
        let mut left = left.into();
        if !left.ends_with("<String, ") {
            left.push_str("<String, ")
        }

        Self::new(left, ">")
    }

    pub fn tuple() -> Self {
        Self::new("(", ")")
    }

    pub fn option() -> Self {
        Self::new("Option<", ">")
    }

    pub fn apply(&self, item: String) -> String {
        match self {
            Wrapper::Bottom { left, right } => {
                if left.is_empty() && right.is_empty() {
                    return item;
                }
                format!("{}{}{}", left, item, right)
            }
            Wrapper::Nested { left, right } => left.apply(right.apply(item)),
        }
    }
}

impl Default for Wrapper {
    fn default() -> Self {
        Self::new("", "")
    }
}
