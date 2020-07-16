// #[cfg(test)]
// fn run_it(data: &str) -> Program {
//     let opts = GenerateOptions {
//         json_name: Some("baz".into()),
//         root_name: "Foo".into(),
//         make_unit_test: false,
//         make_main: false,
//         max_size: Some(30),
//         tuple_max: Some(3),
//     };
//     eprintln!("> {}", data);
//     Program::generate(serde_json::from_str(data).unwrap(), opts)
// }

// #[test]
// fn tuple() {
//     let input = r#"[1,false]"#;
//     eprintln!("{}", run_it(input));
// }

// #[test]
// fn struct_() {
//     let input = r#"{"a": 1, "b": false, "c": {"d": 1}}"#;
//     eprintln!("{}", run_it(input));
// }

// #[test]
// fn foo_foo() {
//     let input = r#"{"Foo": {"Foo": {"Foo2": 1}}}"#;
//     eprintln!("{}", run_it(input));
// }

// #[test]
// fn repeated() {
//     let input = r#"{"Foo": {"Foo":1}}"#;
//     eprintln!("{}", run_it(input));
// }

// // #[test]
// // fn serde() {
// //     use serde::{Deserialize, Serialize};

// //     #[derive(Serialize, Deserialize, Debug)]
// //     pub struct Foo {
// //         #[serde(rename = "Foo")]
// //         pub foo: Foo_2,
// //     }

// //     #[derive(Serialize, Deserialize, Debug)]
// //     // #[serde(rename = "Foo")]
// //     pub struct Foo_2 {
// //         #[serde(rename = "Foo")]
// //         pub foo: i64,
// //     }

// //     let input = r#"{"Foo": {"Foo":1}}"#;

// //     let foo = serde_json::from_str::<Foo>(input).unwrap();

// //     assert_eq!(foo.foo.foo, 1);
// // }
