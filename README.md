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

    -l, --large-struct      unroll Objects under this key length
                            - for large objects, if the length is this or smaller
                            - a new struct with all possible (seen) fields will be created


    -t, --max-tuple         heterogeneous arrays under this size will be treated as a tuple
                            - for types such as [1, false, "foo"] if the length exceeds the provided value
                            - then a Vec<Value> will be created instead. otherwise a tuple will be created. 
                            - for the example above: a tuple of (i64, bool, String)

    -v, --version           show the current version
    -h, --help              show this message
```

License: 0BSD
