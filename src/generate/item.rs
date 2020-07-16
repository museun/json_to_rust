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

// TODO replace this with a dump method that writes directly to an
// std::io::Write instead of allocating a string
impl std::fmt::Display for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const STD_DERIVES: &str = "#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]";
        const SERDE_DERIVES: &str = "#[derive(serde::Serialize, serde::Deserialize)]";

        f.write_str(STD_DERIVES)?;
        f.write_str("\n")?;

        f.write_str(SERDE_DERIVES)?;
        f.write_str("\n")?;

        if let Some(rename) = &self.rename {
            writeln!(f, "#[serde(rename = \"{}\")]", rename)?;
        }

        writeln!(f, "pub struct {} {{", self.name)?;

        let fields = {
            let mut f = self.fields.clone();
            f.sort_by(|l, r| l.binding.cmp(&r.binding));
            f
        };

        for field in fields {
            if let Some(rename) = &field.rename {
                writeln!(f, "    #[serde(rename = \"{}\")]", rename)?;
            }
            writeln!(f, "    pub {}: {},", field.binding, field.kind)?;
        }

        writeln!(f, "}}")
    }
}

#[derive(Debug)]
pub struct Item {
    pub ident: String,
    pub body: Vec<String>,
}

// TODO replace this with a dump method that writes directly to an
// std::io::Write instead of allocating a string
impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)?;
        self.body.iter().map(|el| write!(f, "{}", el)).collect()
    }
}
