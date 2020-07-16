use std::io::{BufReader, Read};

mod infer;
mod util;

mod generate;
use generate::Program;

pub fn generate<R: Read + ?Sized>(opts: GenerateOptions, read: &mut R) -> anyhow::Result<String> {
    let mut reader = BufReader::new(read);
    let val: serde_json::Value = serde_json::from_reader(&mut reader)?;
    Ok(Program::generate(val, opts).to_string())
}

#[derive(Debug, Default)]
pub struct GenerateOptions {
    pub json_name: Option<String>,
    pub root_name: String,
    pub make_unit_test: bool,
    pub make_main: bool,
    pub max_size: Option<usize>,
    pub tuple_max: Option<usize>,
}
