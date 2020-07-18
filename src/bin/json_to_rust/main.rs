use inflections::Inflect as _;
use json_to_rust::{all_std_derives, custom, no_derives, CasingScheme, MapWrapper, VecWrapper};

fn header() {
    println!("{}: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

fn print_version() -> ! {
    header();
    std::process::exit(0)
}

fn print_short_help() -> ! {
    const HELP_MSG: &str = r#"
description:
    pipe some json to this and it'll generate some rust for you.

usage:
    < foo.json | json_to_rust -j json_object -n MyStruct > out.rs

flags:
    -u, --make-unit-tests   generate unit tests
    -m, --make-main         generate a main function

    -j, --json-root-name    the name of the root JSON object
    -n, --rust-root-name    the name of the root Rust object

    -t, --max-tuple         heterogeneous arrays under this size will be treated as a tuple

    -d, --derive            add this derive to the generate types
    -nd, --no-std-derives   only use the serde derives

    -f, --field-naming      the casing scheme to use for fields
    -s, --struct-naming     the casing scheme to use for structs

    --vec-wrapper           use this type for Vecs, defaults to 'Vec'
    --map-wrapper           use this type for Maps, defaults to 'HashMap'

    -v, --version           show the current version
    -h, --help              show this message
    "#;

    header();
    println!("{}", HELP_MSG);
    std::process::exit(0)
}

fn print_long_help() -> ! {
    const HELP_MSG: &str = r#"
description:
    pipe some json to this and it'll generate some rust for you.

usage:
    < foo.json | json_to_rust -j json_object -n MyStruct > out.rs

flags:
    -u, --make-unit-tests   generate unit tests
                            - this generates a unit test that round trips the
                            - serialization along with the included json sample

    -m, --make-main         generate a main function
                            - this generates a main function demoing the
                            - serialized and deserialized forms

    -j, --json-root-name    the name of the root JSON object
                            - this takes a string which'll be the name of
                            - the root json object, if applicable.

    -n, --rust-root-name    the name of the root Rust object
                            - this is the name of your root Rust struct.
                            - if not provided, its inferred from the json name

    -t, --max-tuple         heterogeneous arrays under this size will be treated as a tuple
                            - for types such as [1, false, "foo"] if the length exceeds the provided value
                            - then a Vec<Value> will be created instead. otherwise a tuple will be created.
                            - for the example above: a tuple of (i64, bool, String)

    -d, --derive            add this derive to the generate types
                            - this can accept a string or a comma seperated string.
                            - this flag can be used multiple times
                            - the order of the flag is the order of the derives, left to right
                            - it will dedup the list for you
                            - 'Serialize' and 'Deserialize' will be added to the end
                            - if this nor [-d, --derive] are provided then the full range of std derives will be used

    -nd, --no-std-derives   only use the serde derives
                            - this just uses 'Serialize' and 'Deserialize'
                            - if this nor [-d, --derive] are provided then the full range of std derives will be used

    -f, --field-naming      the casing scheme to use for fields
                            - this default to snake_case
                            - available options [snake, constant, pascal, camel]

    -s, --struct-naming     the casing scheme to use for structs
                            - this defaults to PascalCase
                            - available options [snake, constant, pascal, camel]

    --vec-wrapper           use this type for Vecs, defaults to 'Vec'
    --map-wrapper           use this type for Maps, defaults to 'HashMap'

    -v, --version           show the current version
    -h, --help              show this message
        "#;

    header();
    println!("{}", HELP_MSG);
    std::process::exit(0)
}

fn parse_casing(input: &str) -> Result<CasingScheme, pico_args::Error> {
    let ok = match input.to_lower_case().as_str() {
        "snake" => CasingScheme::Snake,
        "pascal" => CasingScheme::Pascal,
        "constant" => CasingScheme::Constant,
        "camel" => CasingScheme::Camel,
        s => {
            let cause = format!("'{}' unknown casing. try [snake,pascal,constant,camel]", s);
            let err = pico_args::Error::ArgumentParsingFailed { cause };
            return Err(err);
        }
    };
    Ok(ok)
}

fn parse_args() -> anyhow::Result<json_to_rust::Options> {
    let mut args = pico_args::Arguments::from_env();

    match (
        args.contains(["-v", "--version"]),
        args.contains("-h"),
        args.contains("--help"),
    ) {
        (true, _, _) => print_version(),
        (_, true, _) => print_short_help(),
        (_, _, true) => print_long_help(),
        _ => {}
    }

    let json_name = args.opt_value_from_str(["-j", "--json-root-name"])?;

    let opts = json_to_rust::Options {
        make_unit_test: args.contains(["-u", "--make-unit-tests"]),
        make_main: args.contains(["-m", "--make-main"]),

        tuple_max: args.opt_value_from_str(["-t", "--max-tuple"])?,

        root_name: args
            .opt_value_from_str(["-n", "--rust-root-name"])?
            .or_else(|| json_name.as_ref().map(|s: &String| s.to_pascal_case()))
            .unwrap_or_else(|| "MyRustStruct".into()),

        default_derives: if args.contains(["-nd", "--no-std-derives"]) {
            no_derives()
        } else {
            match args
                .values_from_str::<_, String>(["-d", "--derive"])?
                .as_slice()
            {
                [] => all_std_derives(),
                [list @ ..] => custom(list),
            }
        },

        json_name,

        field_naming: args
            .opt_value_from_fn(["-f", "--field-naming"], parse_casing)?
            .unwrap_or_else(|| CasingScheme::Snake),

        struct_naming: args
            .opt_value_from_fn(["-s", "--struct-naming"], parse_casing)?
            .unwrap_or_else(|| CasingScheme::Pascal),

        vec_wrapper: args
            .opt_value_from_str("--vec-wrapper")?
            .map(VecWrapper::custom)
            .unwrap_or_else(VecWrapper::std),

        map_wrapper: args
            .opt_value_from_str("--map-wrapper")?
            .map(MapWrapper::custom)
            .unwrap_or_else(MapWrapper::std),
    };

    args.finish()?;

    Ok(opts)
}

fn main() -> anyhow::Result<()> {
    let opts = parse_args()?;

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut out = std::io::stdout();
    json_to_rust::generate(opts, &mut stdin, &mut out)
}
