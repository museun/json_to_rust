use super::Shape;

#[derive(PartialOrd, PartialEq, Ord, Eq, Debug)]
pub enum Local {
    Bool,
    Integer,
    Float,
    String,
    Optional(Box<Self>),
    Array(Box<Self>),
    Complex,
}

impl Local {
    pub fn new(shape: Shape) -> Self {
        match shape {
            Shape::Optional(ty) => Self::Optional(Box::new(Self::new(*ty))),

            Shape::Bool => Self::Bool,
            Shape::String => Self::String,
            Shape::Integer => Self::Integer,
            Shape::Float => Self::Float,
            Shape::Tuple(el, ..) => Self::new(Shape::fold(el)),
            Shape::Array(ty) => Self::Array(Box::new(Self::new(*ty))),

            _ => Self::Complex,
        }
    }

    pub fn format(self, s: &mut String) {
        const ANY_VALUE: &str = "::serde_json::Value";

        match self {
            Self::Bool => s.push_str("bool"),
            Self::Integer => s.push_str("i64"),
            Self::Float => s.push_str("f64"),
            Self::String => s.push_str("String"),
            Self::Array(ty) => {
                s.push_str("::std::vec::Vec<");
                Self::format(*ty, s);
                s.push('>')
            }
            Self::Optional(ty) => {
                s.push_str("::std::option::Option<");
                Self::format(*ty, s);
                s.push('>')
            }
            Self::Complex => s.push_str(ANY_VALUE),
        }
    }
}
