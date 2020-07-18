use super::item::{Field, Item, Struct};
use crate::{
    infer::{self, Map, Shape},
    util, CasingScheme, Options,
};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Generator<'a> {
    pub structs: Vec<Struct>,
    pub items: Vec<Item>,
    pub opts: &'a Options,

    pub seen_structs: HashSet<String>,
    pub depth: usize,

    pub root_at: usize,
    pub wrap_in_vec: Option<Struct>,
}

impl<'a> Generator<'a> {
    pub fn new(opts: &'a Options) -> Self {
        let (structs, items, seen_structs, depth, wrap_in_vec) = <_>::default();

        Self {
            opts,
            root_at: 1,

            structs,
            items,
            seen_structs,
            depth,
            wrap_in_vec,
        }
    }

    const ANY_VALUE: &'static str = "::serde_json::Value";

    pub fn walk(&mut self, shape: &Shape, wrap: Wrapper, name: &str) {
        if self.depth == 0
            && (matches!(shape, Shape::Array(..)) | matches!(shape, Shape::Tuple(..)))
        {
            // we're at the root and its an array so we should generate a Vec<Struct>
            let t = self.wrap_in_vec.replace(Struct {
                rename: None,
                name: format!("{}List", self.opts.root_name),
                fields: vec![Field {
                    rename: self.opts.json_name.clone(),
                    binding: "list".into(),
                    kind: VEC_WRAPPER.apply(self.opts.root_name.clone()),
                }],
            });
            assert!(
                t.is_none(),
                "only a top-level array should be provided. NDJSON is not supported"
            );

            self.root_at += 1;
        }

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

            Shape::Tuple(els, ..) => {
                let folded = Shape::fold(els.clone());
                // eprintln!("folded: [{}; {}]", folded.root(), e);
                if folded == Shape::Any && els.iter().any(|s| *s != Shape::Any) {
                    self.make_tuple(els, NOOP_WRAPPER)
                } else {
                    self.make_vec(&folded, name)
                }
            }

            Shape::Object(ty) => self.make_struct(name, ty, NOOP_WRAPPER),
        }

        self.depth -= 1;
    }

    fn make_tuple(&mut self, shapes: &[Shape], wrap: Wrapper) {
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
            ident: TUPLE_WRAPPER.apply(types),
            body: defs,
        });
    }

    fn make_struct(&mut self, input_name: &str, map: &Map, wrap: Wrapper) {
        let struct_naming = if self.depth == 1 {
            CasingScheme::Identity
        } else {
            self.opts.struct_naming
        };

        let struct_name = util::fix_name(input_name, &mut self.seen_structs, struct_naming);

        let mut defs = Vec::new();
        let mut body = Vec::new();

        let mut seen_fields = HashSet::new();

        for (name, shape) in map.iter().rev() {
            let field_name = util::fix_name(name, &mut seen_fields, self.opts.field_naming);
            let field_renamed = field_name != *name;

            match shape {
                // flatten big structs into just a hashmap (TODO unify the maps
                // and use a metric for determining how many fields are
                // acceptable)
                Shape::Object(map) => self.make_field_map(map),
                Shape::Map(_) => unreachable!("shouldn't have a map here"),
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
                .filter(|_| self.wrap_in_vec.is_none() && self.depth == self.root_at)
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
        self.walk(ty, MAP_WRAPPER, "");
    }

    fn make_vec(&mut self, ty: &Shape, name: &str) {
        self.walk(ty, VEC_WRAPPER, name);
    }

    fn write_primitive(&mut self, s: impl Into<String>, wrap: Wrapper) {
        let s = s.into();
        self.items.push(Item {
            ident: wrap.apply(s),
            body: vec![],
        });
    }
}

pub static MAP_WRAPPER: Wrapper = Wrapper {
    left: "::std::collections::HashMap<String, ",
    right: ">",
};

pub static VEC_WRAPPER: Wrapper = Wrapper {
    left: "::std::vec::Vec<",
    right: ">",
};

pub static TUPLE_WRAPPER: Wrapper = Wrapper {
    left: "(",
    right: ")",
};

pub static NOOP_WRAPPER: Wrapper = Wrapper::new();

#[derive(Copy, Clone)]
pub struct Wrapper {
    left: &'static str,
    right: &'static str,
}

impl Wrapper {
    pub const fn new() -> Self {
        Self {
            left: "",
            right: "",
        }
    }

    pub fn apply(&self, item: String) -> String {
        if self.left.is_empty() && self.right.is_empty() {
            return item;
        }
        format!("{}{}{}", self.left, item, self.right)
    }
}
