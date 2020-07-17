use crate::CasingScheme;
use std::collections::HashSet;

pub const KEYWORDS: &[&str] = &[
    "abstract", "alignof", "as", "become", "box", "break", "const", "continue", "crate", "do",
    "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in", "let", "loop",
    "macro", "match", "mod", "move", "mut", "offsetof", "override", "priv", "proc", "pub", "pure",
    "ref", "return", "Self", "self", "sizeof", "static", "struct", "super", "trait", "true",
    "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield", "async",
    "await", "try",
];

// pub fn fix_struct_name(name: &str, casing: CasingScheme, used: &mut HashSet<String>) -> String {
//     fix_name(name, used, casing)
// }

// pub fn fix_field_name(name: &str, casing: CasingScheme, used: &mut HashSet<String>) -> String {
//     fix_name(name, used, casing)
// }

pub fn fix_name(name: &str, used: &mut HashSet<String>, casing: CasingScheme) -> String {
    let name = name.trim();
    let mut out = match name.chars().next() {
        Some(c) if c.is_ascii() && c.is_numeric() => casing.convert(&format!("n{}", name)),
        _ => casing.convert(name),
    };

    if KEYWORDS.contains(&&*out) {
        out.push_str("_");
    }

    assert!(!out.is_empty());

    if !used.contains(&out) {
        used.insert(out.clone());
        return out;
    }

    for i in 2.. {
        let temp = format!("{}_{}", out, i);
        if !used.contains(&temp) {
            used.insert(temp.clone());
            return temp;
        }
    }

    unreachable!()
}
