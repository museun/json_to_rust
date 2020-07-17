use anyhow::Context as _;
use inflections::Inflect as _;
use json_to_rust::{all_std_derives, custom, no_derives};

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

    -l, --large-struct      unroll Objects under this key length
    -t, --max-tuple         heterogeneous arrays under this size will be treated as a tuple

    -d, --derive            add this derive to the generate types
    -nd, --no-std-derives   only use the serde derives

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

    -l, --large-struct      unroll Objects under this key length
                            - for large objects, if the length is this or smaller
                            - a new struct with all possible (seen) fields will be created

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

    -v, --version           show the current version
    -h, --help              show this message
        "#;

    header();
    println!("{}", HELP_MSG);
    std::process::exit(0)
}

fn main() -> anyhow::Result<()> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-v", "--version"]) {
        print_version();
    }

    if args.contains("-h") {
        print_short_help();
    }

    if args.contains("--help") {
        print_long_help();
    }

    let opts = {
        let mut opts = json_to_rust::Options::default();

        opts.make_unit_test = args.contains(["-u", "--make-unit-tests"]);
        opts.make_main = args.contains(["-m", "--make-main"]);

        opts.tuple_max = args.opt_value_from_str(["-t", "--max-tuple"])?;
        opts.max_size = args.opt_value_from_str(["-l", "--large-struct"])?;

        opts.json_name = args.opt_value_from_str(["-j", "--json-root-name"])?;

        opts.root_name = args
            .opt_value_from_str(["-n", "--rust-root-name"])?
            .or_else(|| opts.json_name.as_ref().map(|s| s.to_pascal_case()))
            .with_context(|| {
                "`[-n, --rust-root-name]` is required if `[-j, --json-root-name]` is not provided"
            })?;

        opts.default_derives = if args.contains(["-nd", "--no-std-derives"]) {
            no_derives()
        } else {
            match args
                .values_from_str::<_, String>(["-d", "--derive"])?
                .as_slice()
            {
                [] => all_std_derives(),
                [list @ ..] => custom(list),
            }
        };

        if opts.default_derives.is_empty() {
            opts.default_derives = no_derives()
        }

        args.finish()?;

        opts
    };

    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let mut out = std::io::stdout();
    json_to_rust::generate(opts, &mut stdin, &mut out)
}
