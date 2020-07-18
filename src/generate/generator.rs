use super::item::{Field, Item, Struct};
use crate::{
    infer::{self, Map, Shape},
    util, CasingScheme, Options,
};
use std::collections::HashSet;
use util::Wrapper;

#[derive(Debug)]
pub struct Generator<'a> {
    pub structs: Vec<Struct>,
    pub items: Vec<Item>,
    pub opts: &'a Options,

    pub seen_structs: HashSet<String>,
    pub depth: usize,

    pub should_include_map: bool,

    pub root_at: usize,
    pub wrap_in_vec: Option<Struct>,
}

impl<'a> Generator<'a> {
    pub fn new(opts: &'a Options) -> Self {
        let (structs, items, seen_structs, depth, wrap_in_vec) = <_>::default();

        Self {
            structs,
            items,
            opts,

            seen_structs,
            depth,

            should_include_map: false,

            root_at: 1,
            wrap_in_vec,
        }
    }

    const ANY_VALUE: &'static str = "::serde_json::Value";

    pub fn walk(&mut self, shape: &Shape, wrap: &Wrapper, name: &str, default: &mut bool) {
        if self.depth == 0
            && (matches!(shape, Shape::Array(..)) | matches!(shape, Shape::Tuple(..)))
        {
            // we're at the root and its an array so we should generate a Vec<Struct>
            let t = self.wrap_in_vec.replace(Struct {
                rename: None,
                name: format!("{}List", self.opts.root_name),
                fields: vec![Field {
                    rename: self.opts.json_name.clone(),
                    default: *default,
                    binding: "list".into(),
                    kind: self.opts.vec_wrapper.apply(self.opts.root_name.clone()),
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
            Shape::Optional(inner) => {
                let wrap = Wrapper::wrap(wrap.clone(), Wrapper::option());
                self.walk(inner, &wrap, name, default)
            }
            Shape::Array(ty) => self.make_vec(ty, name, wrap, default),
            Shape::Map(ty) => self.make_map(ty, default),

            Shape::Tuple(els, ..) => {
                let folded = Shape::fold(els.clone());
                // eprintln!("folded: [{}; {}]", folded.root(), e);
                if folded == Shape::Any && els.iter().any(|s| *s != Shape::Any) {
                    self.make_tuple(els, &Wrapper::default(), default)
                } else {
                    self.make_vec(&folded, name, wrap, default)
                }
            }

            Shape::Object(ty) => self.make_struct(name, ty, &Wrapper::default(), default),
        }

        self.depth -= 1;
    }

    fn make_tuple(&mut self, shapes: &[Shape], wrap: &Wrapper, default: &mut bool) {
        let (mut types, mut defs) = (String::new(), Vec::new());
        for shape in shapes {
            self.walk(shape, wrap, "", default);
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
            ident: Wrapper::tuple().apply(types),
            body: defs,
        });
    }

    fn make_struct(&mut self, input_name: &str, map: &Map, wrap: &Wrapper, default: &mut bool) {
        let struct_naming = if self.depth == 1 {
            CasingScheme::Identity
        } else {
            self.opts.struct_naming
        };

        let struct_name = util::fix_name(input_name, &mut self.seen_structs, struct_naming);

        let mut defs = Vec::new();
        let mut body = Vec::new();

        let mut seen_fields = HashSet::new();

        fn collapse_option_vec(shape: &Shape, should_collapse: bool) -> Option<&Shape> {
            if should_collapse {
                if let Shape::Optional(ty) = shape {
                    if let Shape::Array(..) = **ty {
                        return Some(ty);
                    }
                }
            }
            None
        }

        for (name, shape) in map.iter().rev() {
            let field_name = util::fix_name(name, &mut seen_fields, self.opts.field_naming);
            let field_renamed = field_name != *name;

            match shape {
                // flatten big structs into just a hashmap (TODO unify the maps
                // and use a metric for determining how many fields are
                // acceptable)
                Shape::Object(map) => self.make_field_map(map),
                Shape::Map(_) => unreachable!("shouldn't have a map here"),
                _ => {
                    if let Some(shape) = collapse_option_vec(shape, self.opts.collapse_option_vec) {
                        // mark it as default
                        *default = true;
                        self.walk(shape, wrap, &field_name, default)
                    } else {
                        self.walk(shape, wrap, &field_name, default)
                    }
                }
            }

            let item = self.items.pop().unwrap();
            defs.extend(item.body);

            body.push(Field {
                rename: if field_renamed {
                    Some(name.clone())
                } else {
                    None
                },
                default: *default,
                binding: field_name,
                kind: item.ident,
            });
            *default = false;
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

        let mut ident = String::new();
        local.format(&mut ident, &self.opts);
        let ident = self.opts.map_wrapper.apply(ident);
        self.should_include_map = true;

        self.items.push(Item {
            ident,
            body: vec![],
        })
    }

    fn make_map(&mut self, ty: &Shape, default: &mut bool) {
        self.walk(ty, &self.opts.map_wrapper, "", default);
        self.should_include_map = true;
    }

    fn make_vec(&mut self, ty: &Shape, name: &str, wrap: &Wrapper, default: &mut bool) {
        self.walk(
            ty,
            &wrap.clone().wrap(self.opts.vec_wrapper.clone()),
            name,
            default,
        );
    }

    fn write_primitive(&mut self, s: impl Into<String>, wrap: &Wrapper) {
        let s = s.into();
        self.items.push(Item {
            ident: wrap.apply(s),
            body: vec![],
        });
    }
}
