use super::Print;
use crate::Options;

#[derive(Debug)]
pub struct Struct {
    pub rename: Option<String>,
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub rename: Option<String>,
    pub binding: String,
    pub kind: String,
}

impl Print for Struct {
    fn print<W: std::io::Write + ?Sized>(&self, writer: &mut W, opts: &Options) -> super::IoResult {
        writeln!(writer, "#[derive({})]", &opts.default_derives)?;

        if let Some(rename) = &self.rename {
            writeln!(writer, "#[serde(rename = \"{}\")]", rename)?;
        }

        writeln!(writer, "pub struct {} {{", self.name)?;

        let fields = {
            let mut fields = self.fields.clone();
            fields.sort_by(|l, r| l.binding.cmp(&r.binding));
            fields
        };

        for field in fields {
            if let Some(rename) = &field.rename {
                writeln!(writer, "    #[serde(rename = \"{}\")]", rename)?;
            }
            writeln!(writer, "    pub {}: {},", field.binding, field.kind)?;
        }

        writeln!(writer, "}}")
    }
}

#[derive(Debug)]
pub struct Item {
    pub ident: String,
    pub body: Vec<String>,
}

impl Print for Item {
    fn print<W: std::io::Write + ?Sized>(&self, writer: &mut W, _: &Options) -> super::IoResult {
        write!(writer, "{}", self.ident)?;
        self.body
            .iter()
            .map(|el| write!(writer, "{}", el))
            .collect()
    }
}
