```
json_to_rust: 0.1.0

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

    --flatten-option-vec    flattens Option<Vec<T>> into just Vec<T>
                            - this also uses serde_default which'll create an empty Vec if it was None

    -v, --version           show the current version
    -h, --help              show this message
```

License: 0BSD
