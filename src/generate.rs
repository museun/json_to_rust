mod generator;
pub mod item;

mod program;
pub use program::Program;

use crate::Options;
use std::io::{self, Write};

pub type IoResult = io::Result<()>;

pub trait Print {
    fn print<W: Write + ?Sized>(&self, writer: &mut W, options: &Options) -> IoResult;
}
