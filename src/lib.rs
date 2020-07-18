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
#[derive(Debug)]
pub struct Options {
    pub json_name: Option<String>,
    pub root_name: String,
    pub make_unit_test: bool,
    pub make_main: bool,
    pub max_size: Option<usize>,
    pub tuple_max: Option<usize>,

    pub default_derives: String,
    pub field_naming: CasingScheme,
    pub struct_naming: CasingScheme,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum CasingScheme {
    Snake,
    Pascal,
    Constant,
    Camel,
}

impl CasingScheme {
    fn convert(self, input: &str) -> String {
        use inflections::Inflect as _;
        match self {
            Self::Snake => input.to_snake_case(),
            Self::Pascal => input.to_pascal_case(),
            Self::Constant => input.to_constant_case(),
            Self::Camel => input.to_camel_case(),
        }
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

            field_naming: CasingScheme::Snake,
            struct_naming: CasingScheme::Pascal,
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
    const DEFAULT: [&str; 2] = ["Serialize", "Deserialize"];

    list.into_iter()
        .flat_map(|s| {
            s.to_string()
                .split(',') // split if the user provided a comma-separated list
                .filter(|c| !c.starts_with(char::is_numeric)) // invalid trait name (TODO: make this more rigid)
                .map(ToString::to_string)
                .collect::<IndexSet<_>>() // de-dupe
        })
        .filter(|s| !DEFAULT.contains(&&**s)) // remove defaults if they are in the middle
        .chain(DEFAULT.iter().map(ToString::to_string)) // append defaulfs to the end
        .collect::<IndexSet<_>>() // de-dupe
        .into_iter()
        .fold(String::new(), |mut a, c| {
            if !a.is_empty() {
                a.push_str(", ");
            }
            a.push_str(&c);
            a
        })
}
