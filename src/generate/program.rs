use super::generator::Generator;
use crate::{infer::Shape, Options};
use serde_json::Value;

pub struct Program {
    pub body: String,
    pub is_snippet: bool,
}

impl Program {
    pub fn generate(val: Value, opts: Options) -> Self {
        let root_name = opts.root_name.clone();
        let tuple_max = opts.tuple_max;
        let mut g = Generator {
            opts,
            ..Generator::default()
        };
        g.walk(
            &Shape::new(&val, tuple_max.unwrap_or_default()),
            None,
            &root_name,
        );

        let (mut items, mut structs);
        let iter: &mut dyn Iterator<Item = String> = if g.structs.is_empty() {
            items = g.items.iter().map(|s| s.to_string());
            &mut items
        } else {
            structs = g.structs.iter().rev().map(|s| s.to_string());
            &mut structs
        };

        let body = iter.fold(String::new(), |mut a, c| {
            if !a.is_empty() {
                a.push('\n');
            }
            a.push_str(&c);
            a
        });

        Self {
            body,
            is_snippet: g.structs.is_empty(),
        }
    }
}

// TODO replace this with a dump method that writes directly to an
// std::io::Write instead of allocating a string
impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_snippet {
            write!(f, "// ")?;
        }
        write!(f, "{}", self.body)
    }
}
