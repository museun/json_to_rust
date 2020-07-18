use indexmap::IndexSet;
use std::io::{BufReader, BufWriter, Read, Write};

mod infer;
mod util;
use util::Wrapper;

mod generate;
use generate::{Print, Program};

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

#[derive(Debug, Default)]
pub struct VecWrapper(pub Wrapper);

impl VecWrapper {
    pub fn std() -> Self {
        Self(Wrapper {
            left: "Vec<",
            right: ">",
        })
    }

    pub fn custom(mut left: String) -> Self {
        if !left.ends_with('<') {
            left.push('<')
        }
        Self(Wrapper::from_string(left))
    }
}

#[derive(Debug, Default)]
pub struct MapWrapper(pub Wrapper);

impl MapWrapper {
    pub fn std() -> Self {
        Self(Wrapper {
            left: "HashMap<String, ",
            right: ">",
        })
    }

    pub fn custom(mut left: String) -> Self {
        if !left.ends_with("<String, ") {
            left.push_str("<String, ")
        }
        Self(Wrapper::from_string(left))
    }
}

#[derive(Debug)]
pub struct Options {
    pub json_name: Option<String>,
    pub root_name: String,

    pub make_unit_test: bool,
    pub make_main: bool,

    pub tuple_max: Option<usize>,

    pub default_derives: String,
    pub field_naming: CasingScheme,
    pub struct_naming: CasingScheme,

    pub vec_wrapper: VecWrapper,
    pub map_wrapper: MapWrapper,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum CasingScheme {
    Snake,
    Pascal,
    Constant,
    Camel,
    Identity,
}

impl CasingScheme {
    fn convert(self, input: &str) -> String {
        use inflections::Inflect as _;
        match self {
            Self::Snake => input.to_snake_case(),
            Self::Pascal => input.to_pascal_case(),
            Self::Constant => input.to_constant_case(),
            Self::Camel => input.to_camel_case(),
            Self::Identity => input.to_string(),
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
        .chain(DEFAULT.iter().map(ToString::to_string)) // append defaults to the end
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
