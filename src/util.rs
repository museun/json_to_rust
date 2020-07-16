use inflections::Inflect as _;

use std::collections::HashSet;

pub(crate) const KEYWORDS: &[&str] = &[
    "abstract", "alignof", "as", "become", "box", "break", "const", "continue", "crate", "do",
    "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in", "let", "loop",
    "macro", "match", "mod", "move", "mut", "offsetof", "override", "priv", "proc", "pub", "pure",
    "ref", "return", "Self", "self", "sizeof", "static", "struct", "super", "trait", "true",
    "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield", "async",
    "await", "try",
];

pub(crate) fn fix_struct_name(name: &str, used: &mut HashSet<String>) -> String {
    fix_name(name, used, to_pascal_case)
}

pub(crate) fn fix_field_name(name: &str, used: &mut HashSet<String>) -> String {
    fix_name(name, used, to_snake_case)
}

fn fix_name(name: &str, used: &mut HashSet<String>, rename: fn(&str) -> String) -> String {
    let name = name.trim();
    let mut out = match name.chars().next() {
        Some(c) if c.is_ascii() && c.is_numeric() => rename(&format!("n{}", name)),
        _ => rename(name),
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

pub(crate) fn to_snake_case(name: &str) -> String {
    name.to_snake_case()

    // let mut out = String::with_capacity(name.len());
    // let mut seen = false;
    // for (i, ch) in name.char_indices() {
    //     if i == 0 && ch == '_' {
    //         continue;
    //     }

    //     if i > 0 && ch.is_numeric() && !seen {
    //         out.push('_');
    //         seen = true;
    //     }

    //     if !ch.is_numeric() {
    //         seen = false
    //     }

    //     if ch == '-' {
    //         out.push('_');
    //         seen = true;
    //     } else {
    //         out.push(ch.to_ascii_lowercase());
    //     }
    // }
    // out
}

pub(crate) fn to_pascal_case(name: &str) -> String {
    name.to_pascal_case()

    // let name = name.trim();
    // let mut out = String::with_capacity(name.len());
    // let mut upper = true;
    // for ch in name.chars() {
    //     if ch == '_' {
    //         upper = true
    //     } else if upper {
    //         out.push(ch.to_ascii_uppercase());
    //         upper = false
    //     } else {
    //         out.push(ch)
    //     }
    // }
    // assert!(!out.is_empty());
    // out
}
