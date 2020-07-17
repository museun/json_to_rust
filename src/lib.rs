use std::io::{BufReader, BufWriter, Read, Write};

mod infer;
mod util;

mod generate;
use generate::{Print, Program};
use indexmap::IndexSet;

pub fn generate<R, W>(opts: Options, read: &mut R, write: &mut W) -> anyhow::Result<()>
where
    R: Read + ?Sized,
    W: Write + ?Sized,
{
    let mut reader = BufReader::new(read);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    let program = Program::generate(json::parse(&buf)?, &buf, &opts);

    let mut writer = BufWriter::new(write);
    program.print(&mut writer, &opts)?;

    Ok(())
}

// TODO document this
pub struct Options {
    pub json_name: Option<String>,
    pub root_name: String,
    pub make_unit_test: bool,
    pub make_main: bool,
    pub max_size: Option<usize>,
    pub tuple_max: Option<usize>,

    pub default_derives: String,
}

impl std::fmt::Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options").finish()
    }
}

impl Default for Options {
    fn default() -> Self {
        let (json_name, make_unit_test, make_main, max_size, tuple_max, default_derives) =
            <_>::default();

        Self {
            root_name: "MyRustStruct".into(),

            json_name,
            make_unit_test,
            make_main,
            max_size,
            tuple_max,
            default_derives,
        }
    }
}

pub fn no_derives() -> String {
    custom::<&str, _>(std::iter::empty())
}

pub fn all_std_derives() -> String {
    custom(&[
        "Clone",
        "Debug",
        "PartialEq",
        "PartialOrd",
        "Eq",
        "Ord",
        "Hash",
    ])
}

pub fn custom<S, L>(list: L) -> String
where
    S: ToString,
    L: IntoIterator<Item = S>,
{
    let default = ["Serialize", "Deserialize"];

    let derives = list
        .into_iter()
        .flat_map(|s| {
            s.to_string()
                .split(',')
                .filter(|c| !c.starts_with(char::is_numeric))
                .map(ToString::to_string)
                .collect::<IndexSet<_>>()
        })
        .filter(|s| !default.contains(&&**s))
        .chain(default.iter().map(ToString::to_string))
        .collect::<IndexSet<_>>();

    derives.into_iter().fold(String::new(), |mut a, c| {
        if !a.is_empty() {
            a.push_str(", ");
        }
        a.push_str(&c);
        a
    })
}
