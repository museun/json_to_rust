use std::io::{BufReader, BufWriter, Read, Write};

mod infer;
mod util;

mod generate;
use generate::Program;

pub fn generate<R: Read + ?Sized, W: Write + ?Sized>(
    opts: GenerateOptions,
    read: &mut R,
    write: &mut W,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(read);
    let val: serde_json::Value = serde_json::from_reader(&mut reader)?;
    let program = Program::generate(val, opts);

    let mut writer = BufWriter::new(write);
    writer.write_all(program.to_string().as_bytes())?;
    Ok(())
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
