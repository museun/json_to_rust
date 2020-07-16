use crate::{
    infer::{self, Map, Shape},
    util, GenerateOptions,
};
use serde_json::Value;
use std::collections::HashSet;

pub struct Program {
    pub body: String,
    pub is_snippet: bool,
}

impl Program {
    pub fn generate(val: Value, opts: GenerateOptions) -> Self {
        let root_name = opts.root_name.clone();
        let tuple_max = opts.tuple_max;
        let mut g = Generated::new(opts);
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

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_snippet {
            write!(f, "// ")?;
        }
        write!(f, "{}", self.body)
    }
}

#[derive(Debug)]
struct Struct {
    rename: Option<String>,
    name: String,
    fields: Vec<Field>,
}

#[derive(Debug, Clone)]
struct Field {
    rename: Option<String>,
    binding: String,
    kind: String,
}

impl std::fmt::Display for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const STD_DERIVES: &str = "#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]";
        const SERDE_DERIVES: &str = "#[derive(serde::Serialize, serde::Deserialize)]";

        f.write_str(STD_DERIVES)?;
        f.write_str("\n")?;

        f.write_str(SERDE_DERIVES)?;
        f.write_str("\n")?;

        if let Some(rename) = &self.rename {
            writeln!(f, "#[serde(rename = \"{}\")]", rename)?;
        }

        writeln!(f, "pub struct {} {{", self.name)?;

        let fields = {
            let mut f = self.fields.clone();
            f.sort_by(|l, r| l.binding.cmp(&r.binding));
            f
        };

        for field in fields {
            if let Some(rename) = &field.rename {
                writeln!(f, "    #[serde(rename = \"{}\")]", rename)?;
            }
            writeln!(f, "    pub {}: {},", field.binding, field.kind)?;
        }

        writeln!(f, "}}")
    }
}

#[derive(Debug)]
struct Item {
    ident: String,
    body: Vec<String>,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)?;
        self.body.iter().map(|el| write!(f, "{}", el)).collect()
    }
}

type Id = Option<fn(&str) -> String>;

#[derive(Debug)]
struct Generated {
    opts: GenerateOptions,
    seen_structs: HashSet<String>,
    structs: Vec<Struct>,
    items: Vec<Item>,

    depth: usize,
}

impl Generated {
    const ANY_VALUE: &'static str = "::serde_json::Value";

    fn new(opts: GenerateOptions) -> Self {
        let (seen_structs, structs, items) = <_>::default();

        Self {
            opts,
            seen_structs,
            structs,
            items,
            depth: 0,
        }
    }

    fn walk(&mut self, shape: &Shape, wrap: Id, name: &str) {
        self.depth += 1;

        match shape {
            Shape::Bottom | Shape::Any | Shape::Null => self.write_primitive(Self::ANY_VALUE, wrap),

            Shape::Bool => self.write_primitive("bool", wrap),
            Shape::String => self.write_primitive("String", wrap),
            Shape::Integer => self.write_primitive("i64", wrap),
            Shape::Float => self.write_primitive("f64", wrap),
            Shape::Opaque(ty) => self.write_primitive(ty, wrap),
            Shape::Optional(inner) => self.walk(inner, wrap, name),
            Shape::Array(ty) => self.make_vec(ty, name),
            Shape::Map(ty) => self.make_map(ty),

            Shape::Tuple(els, _) => {
                let folded = Shape::fold(els.clone());
                // eprintln!("folded: [{}; {}]", folded.root(), e);
                if folded == Shape::Any && els.iter().any(|s| *s != Shape::Any) {
                    self.make_tuple(els, None)
                } else {
                    self.make_vec(&folded, name)
                }
            }

            Shape::Object(ty) => self.make_struct(name, ty, None),
        }

        self.depth -= 1;
    }

    fn make_tuple(&mut self, shapes: &[Shape], wrap: Id) {
        let (mut types, mut defs) = (String::new(), Vec::new());
        for shape in shapes {
            self.walk(shape, wrap, "");
            if !types.is_empty() {
                types.push_str(", ");
            }

            let last = self.items.pop().unwrap();
            types.push_str(&last.ident);

            for last in last.body {
                if !defs.is_empty() {
                    defs.push("\n".into());
                }
                defs.push(last);
            }
        }

        self.items.push(Item {
            ident: format!("({})", types),
            body: defs,
        });
    }

    fn make_struct(&mut self, input_name: &str, map: &Map, wrap: Id) {
        let struct_name = util::fix_struct_name(input_name, &mut self.seen_structs);

        let mut defs = Vec::new();
        let mut body = Vec::new();

        let mut seen_fields = HashSet::new();

        for (name, shape) in map.iter().rev() {
            let field_name = util::fix_field_name(name, &mut seen_fields);
            let field_renamed = field_name != *name;

            match shape {
                Shape::Object(map) => {
                    if self.opts.max_size.is_none() {
                        self.make_field_map(map)
                    } else if self.opts.max_size.filter(|&max| map.len() > max).is_some() {
                        self.make_field_map(map);
                    } else {
                        self.walk(shape, wrap, &field_name)
                    }
                }
                Shape::Map(_) => panic!("shouldn't have a map here"),
                _ => self.walk(shape, wrap, &field_name),
            }

            let item = self.items.pop().unwrap();
            defs.extend(item.body);
            body.push(Field {
                rename: if field_renamed {
                    Some(name.clone())
                } else {
                    None
                },
                binding: field_name,
                kind: item.ident,
            });
        }

        self.structs.push(Struct {
            rename: self
                .opts
                .json_name
                .as_ref()
                .filter(|_| self.depth == 1)
                .map(Clone::clone),
            name: struct_name.clone(),
            fields: body,
        });

        self.items.push(Item {
            ident: struct_name,
            body: defs,
        });
    }

    fn make_field_map(&mut self, map: &Map) {
        let shape = infer::Shape::fold(map.values().cloned());
        let local = infer::Local::new(shape);

        let mut ident = String::from("::std::collections::HashMap<String, ");
        local.format(&mut ident);
        ident.push('>');

        self.items.push(Item {
            ident,
            body: vec![],
        })
    }

    fn make_map(&mut self, ty: &Shape) {
        self.walk(
            ty,
            Some(|s| format!("::std::collections::HashMap<String, {}>", s)),
            "",
        );
    }

    fn make_vec(&mut self, ty: &Shape, name: &str) {
        self.walk(ty, Some(|s| format!("::std::vec::Vec<{}>", s)), name);
    }

    fn write_primitive(&mut self, s: impl Into<String>, wrap: Id) {
        let s = s.into();
        self.items.push(Item {
            ident: wrap.map(|w| w(&s)).unwrap_or_else(|| s),
            body: vec![],
        });
    }
}

#[cfg(test)]
fn run_it(data: &str) -> Program {
    let opts = GenerateOptions {
        json_name: Some("baz".into()),
        root_name: "Foo".into(),
        make_unit_test: false,
        make_main: false,
        max_size: Some(30),
        tuple_max: Some(3),
    };
    eprintln!("> {}", data);
    Program::generate(serde_json::from_str(data).unwrap(), opts)
}

#[test]
fn tuple() {
    let input = r#"[1,false]"#;
    eprintln!("{}", run_it(input));
}

#[test]
fn struct_() {
    let input = r#"{"a": 1, "b": false, "c": {"d": 1}}"#;
    eprintln!("{}", run_it(input));
}

#[test]
fn foo_foo() {
    let input = r#"{"Foo": {"Foo": {"Foo2": 1}}}"#;
    eprintln!("{}", run_it(input));
}

#[test]
fn repeated() {
    let input = r#"{"Foo": {"Foo":1}}"#;
    eprintln!("{}", run_it(input));
}

// #[test]
// fn serde() {
//     use serde::{Deserialize, Serialize};

//     #[derive(Serialize, Deserialize, Debug)]
//     pub struct Foo {
//         #[serde(rename = "Foo")]
//         pub foo: Foo_2,
//     }

//     #[derive(Serialize, Deserialize, Debug)]
//     // #[serde(rename = "Foo")]
//     pub struct Foo_2 {
//         #[serde(rename = "Foo")]
//         pub foo: i64,
//     }

//     let input = r#"{"Foo": {"Foo":1}}"#;

//     let foo = serde_json::from_str::<Foo>(input).unwrap();

//     assert_eq!(foo.foo.foo, 1);
// }
