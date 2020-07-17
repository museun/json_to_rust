use super::{
    generator::{Generator, NOOP_WRAPPER, VEC_WRAPPER},
    item::{Item, Struct},
    Print,
};
use crate::{generate, infer::Shape, util, Options};

use json::JsonValue as Value;
use std::io::Write;

#[derive(Debug)]
pub struct Program<'a> {
    is_snippet: bool,
    items: Vec<Item>,
    structs: Vec<Struct>,
    wrap_in_vec: Option<Struct>,
    opts: &'a Options,
    data: &'a str,
}

impl<'a> Program<'a> {
    pub fn generate(val: Value, data: &'a str, opts: &'a Options) -> Self {
        let root_name = opts.root_name.clone();
        let tuple_max = opts.tuple_max;

        let mut g = Generator::new(opts);
        g.walk(
            &Shape::new(&val, tuple_max.unwrap_or_default()),
            NOOP_WRAPPER,
            &root_name,
        );

        Self {
            is_snippet: g.structs.is_empty(),
            wrap_in_vec: g.wrap_in_vec,
            items: g.items,
            structs: g.structs,
            opts,
            data,
        }
    }

    fn is_wrapped(&self) -> bool {
        self.wrap_in_vec.is_some()
    }

    #[allow(dead_code)]
    fn get_items(&self) -> Option<&[Item]> {
        match self.items.as_slice() {
            [] => None,
            s => Some(s),
        }
    }

    fn get_root(&self) -> Option<&Struct> {
        self.structs.last()
    }

    fn make_name_binding(&self) -> Option<(String, String)> {
        let name = self.get_root().as_ref().map(|s| &s.name)?;

        let mut type_name = name.to_string();
        let binding = util::to_snake_case(&type_name);

        if self.is_wrapped() {
            type_name = VEC_WRAPPER.apply(type_name);
        }

        Some((binding, type_name))
    }

    fn make_unit_test(&self) -> Option<UnitTest<'a>> {
        let (binding, type_name) = self.make_name_binding()?;

        Some(UnitTest {
            binding,
            type_name,
            sample: &self.data,
        })
    }

    fn make_main(&self) -> Option<MainFunction<'a>> {
        let (binding, type_name) = self.make_name_binding()?;

        Some(MainFunction {
            sample: self.data,
            binding,
            type_name,
        })
    }
}

impl<'a> Print for Program<'a> {
    fn print<W: std::io::Write + ?Sized>(&self, writer: &mut W, opts: &Options) -> super::IoResult {
        if self.is_snippet {
            for item in &self.items {
                write!(writer, "// ")?;
                item.print(writer, opts)?;
                writeln!(writer)?;
            }
            return Ok(());
        }

        writeln!(writer, "use ::serde::{{Serialize, Deserialize}};")?;
        writeln!(writer)?;

        if let Some(array) = &self.wrap_in_vec {
            array.print(writer, opts)?;
            writeln!(writer)?;
        }

        for item in self.structs.iter().rev() {
            item.print(writer, opts)?;
            writeln!(writer)?;
        }

        if self.opts.make_unit_test {
            match self.make_unit_test() {
                Some(func) => func.print(writer, opts)?,
                None => eprintln!("WARNING: cannot create unit test, cannot find root struct name"),
            };
        }

        if self.opts.make_main {
            match self.make_main() {
                Some(func) => func.print(writer, opts)?,
                None => {
                    eprintln!("WARNING: cannot create make function, cannot find root struct name")
                }
            };
        }

        Ok(())
    }
}

struct MainFunction<'a> {
    sample: &'a str,
    type_name: String,
    binding: String,
}

impl<'a> Print for MainFunction<'a> {
    fn print<W: Write + ?Sized>(&self, writer: &mut W, _: &Options) -> generate::IoResult {
        let Self {
            sample,
            binding,
            type_name,
        } = self;

        writeln!(
            writer,
            r#####"
fn main() {{
    let sample = r#"
{sample}
    "#;

    let {binding}: {type_name} = serde_json::from_str(&sample).unwrap();
    println!("deserialize: {{:#?}}", {binding});

    let data = serde_json::to_string_pretty(&{binding}).unwrap();
    println!("serialize: {{}}", data);
}}
            "#####,
            sample = sample,
            binding = binding,
            type_name = type_name,
        )
    }
}

struct UnitTest<'a> {
    type_name: String,
    binding: String,
    sample: &'a str,
}

impl<'a> Print for UnitTest<'a> {
    fn print<W: Write + ?Sized>(&self, writer: &mut W, _: &Options) -> generate::IoResult {
        writeln!(
            writer,
            r#####"
#[test]
fn ensure_{binding}_roundtrips() {{
    let t = <{type_name}>::default();
    let j = serde_json::to_string(&t).unwrap();
    let r: {type_name} = serde_json::from_str(&j).unwrap();
    assert_eq!(t, r);
}}

#[test]
fn ensure_{binding}_from_sample() {{
    let sample = r#"
{sample}
    "#;

    let _: {type_name} = serde_json::from_str(&sample).unwrap();    
}}
        "#####,
            binding = self.binding,
            type_name = self.type_name,
            sample = self.sample,
        )
    }
}
