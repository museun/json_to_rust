mod generator;
pub mod item;
mod program;

use crate::Options;
pub use program::Program;
use std::io::{self, Write};

pub(crate) type IoResult = io::Result<()>;

pub trait Print {
    fn print<W: Write + ?Sized>(&self, writer: &mut W, options: &Options) -> IoResult;
}
