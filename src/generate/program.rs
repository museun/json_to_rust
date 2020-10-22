use super::{
    generator::Generator,
    item::{Item, Struct},
    Print,
};
use crate::{generate, infer::Shape, util::Wrapper, CasingScheme, Options};

use json::JsonValue as Value;
use std::io::Write;

#[derive(Debug)]
pub struct Program<'a> {
    items: Vec<Item>,
    structs: Vec<Struct>,
    wrap_in_vec: Option<Struct>,
    opts: &'a Options,
    data: &'a str,

    should_include_map: bool,
}

impl<'a> Program<'a> {
    pub fn generate(val: Value, data: &'a str, opts: &'a Options) -> Self {
        let root_name = opts.root_name.clone();
        let tuple_max = opts.tuple_max;

        let mut g = Generator::new(opts);
        let shape = Shape::new(&val, tuple_max.unwrap_or_default());

        g.walk(&shape, &Wrapper::default(), &root_name, &mut false);

        let Generator {
            structs,
            wrap_in_vec,
            items,
            should_include_map,
            ..
        } = g;

        Self {
            wrap_in_vec,
            items,
            structs,
            opts,
            data,

            should_include_map,
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
        let binding = CasingScheme::Snake.convert(&type_name);

        if self.is_wrapped() {
            type_name = self.opts.vec_wrapper.apply(type_name);
        }

        Some((binding, type_name))
    }

    fn make_unit_test(&self) -> Option<UnitTest<'a>> {
        let (binding, type_name) = self.make_name_binding()?;

        Some(UnitTest {
            binding,
            type_name,
            sample: self.data,
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
        if self.structs.is_empty() {
            for item in &self.items {
                write!(writer, "// ")?;
                item.print(writer, opts)?;
                writeln!(writer)?;
            }
            return Ok(());
        }

        if self.should_include_map {
            writeln!(writer, "use std::collections::HashMap;")?;
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
                    eprintln!("WARNING: cannot create main function, cannot find root struct name")
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
