use serde_json::Value;

pub type HashMap<K, V> = indexmap::IndexMap<K, V>;
pub type Map = HashMap<String, Shape>;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Bottom,
    Any,
    Optional(Box<Self>),
    Null,
    Bool,
    String,
    Integer,
    Float,
    Array(Box<Self>),
    Object(Map),
    Tuple(Vec<Self>, u64),
    Map(Box<Self>),
    Opaque(String),
}

impl Shape {
    pub fn new(val: &Value, max_tuple: usize) -> Self {
        match *val {
            Value::Null => Shape::Null,
            Value::Bool(..) => Shape::Bool,
            Value::Number(ref n) if n.is_i64() => Shape::Integer,
            Value::Number(..) => Shape::Float,
            Value::String(..) => Shape::String,
            Value::Array(ref array) => {
                let len = array.len();
                if len > 1 && len <= max_tuple {
                    Shape::Tuple(
                        array.iter().map(|s| Self::new(s, max_tuple)).collect(),
                        len as _,
                    )
                } else {
                    let ty = array.iter().fold(Shape::Bottom, |left, v| {
                        Shape::factor(left, Self::new(v, max_tuple))
                    });
                    Shape::Array(Box::new(ty))
                }
            }
            Value::Object(ref map) => {
                let fields = map
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::new(v, max_tuple)))
                    .collect();
                Shape::Object(fields)
            }
        }
    }

    pub(crate) fn root(&self) -> &'static str {
        match self {
            Shape::Bottom => "Bottom",
            Shape::Any => "Any",
            Shape::Optional(_) => "Optional",
            Shape::Null => "Null",
            Shape::Bool => "Bool",
            Shape::String => "String",
            Shape::Integer => "Integer",
            Shape::Float => "Float",
            Shape::Array(_) => "Array",
            Shape::Object(_) => "Object",
            Shape::Tuple(_, _) => "Tuple",
            Shape::Map(_) => "Map",
            Shape::Opaque(_) => "Opaque",
        }
    }

    pub(crate) fn fold(shapes: impl IntoIterator<Item = Self>) -> Self {
        shapes.into_iter().fold(Shape::Bottom, Self::factor)
    }

    pub(crate) fn factor(left: Self, right: Self) -> Self {
        if left == right {
            return left;
        }

        // TODO sort tuple ('normalize')
        match (left, right) {
            // equal primitives
            (shape, Self::Bottom) | (Self::Bottom, shape) => shape,

            // promote int|float to float
            (Self::Integer, Self::Float) | (Self::Float, Self::Integer) => Self::Float,

            // nulls are represented by optionals
            (shape, Self::Null) | (Self::Null, shape) => shape.into_optional(),

            // re-factor inner value of optional
            (left, Self::Optional(right)) | (Self::Optional(right), left) => {
                Self::factor(left, *right).into_optional()
            }

            // tuple, tuple
            (Self::Tuple(left, left_arity), Self::Tuple(right, right_arity)) => {
                if left.len() == right.len() {
                    let shapes = left
                        .into_iter()
                        .zip(right.into_iter())
                        .map(|(l, r)| Self::factor(l, r))
                        .collect();
                    Self::Tuple(shapes, left_arity + right_arity)
                } else {
                    Self::Array(Box::new(Self::factor(Self::fold(left), Self::fold(right))))
                }
            }

            // tuple, array | array, tuple
            (Self::Tuple(els, ..), Self::Array(ty)) | (Self::Array(ty), Self::Tuple(els, ..)) => {
                Self::Array(Box::new(Self::factor(*ty, Self::fold(els))))
            }

            // factor array types
            (Self::Array(left), Self::Array(right)) => {
                Self::Array(Box::new(Self::factor(*left, *right)))
            }

            // factor map types
            (Self::Map(left), Self::Map(right)) => Self::Map(Box::new(Self::factor(*left, *right))),

            // factor fields of objects
            (Self::Object(left), Self::Object(right)) => {
                Self::Object(Self::factor_fields(left, right))
            }

            // equal opaque types
            (Self::Opaque(name), ..) | (.., Self::Opaque(name)) => Self::Opaque(name),

            // otherwise anything
            _ => Self::Any,
        }
    }

    fn factor_fields(left: Map, mut right: Map) -> HashMap<String, Self> {
        if left == right {
            return left;
        }

        let mut unified = HashMap::new();
        unified.extend(left.into_iter().map(|(k, v)| {
            let v = match right.remove(&k) {
                Some(r) => Self::factor(v, r),
                None => v.into_optional(),
            };
            (k, v)
        }));
        unified.extend(right.into_iter().map(|(k, v)| (k, v.into_optional())));
        unified
    }

    fn into_optional(self) -> Self {
        match self {
            Shape::Bottom | Shape::Any | Shape::Null | Shape::Optional(_) => self,
            other => Self::Optional(Box::new(other)),
        }
    }
}

#[derive(PartialOrd, PartialEq, Ord, Eq, Debug)]
pub(crate) enum Local {
    Bool,
    Integer,
    Float,
    String,
    Optional(Box<Self>),
    Array(Box<Self>),
    Complex,
}

impl Local {
    pub(crate) fn new(shape: Shape) -> Self {
        match shape {
            Shape::Optional(ty) => Self::Optional(Box::new(Self::new(*ty))),

            Shape::Bool => Self::Bool,
            Shape::String => Self::String,
            Shape::Integer => Self::Integer,
            Shape::Float => Self::Float,
            Shape::Tuple(el, ..) => Local::new(Shape::fold(el)),
            Shape::Array(ty) => Self::Array(Box::new(Local::new(*ty))),

            _ => Self::Complex,
        }
    }

    pub(crate) fn format(self, s: &mut String) {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unify() {
        use Shape::*;
        assert_eq!(Shape::factor(Bool, Bool), Bool);
        assert_eq!(Shape::factor(Bool, Integer), Any);
        assert_eq!(Shape::factor(Integer, Float), Float);
        assert_eq!(Shape::factor(Null, Any), Any);
        assert_eq!(Shape::factor(Null, Bool), Optional(Box::new(Bool)));
        assert_eq!(
            Shape::factor(Null, Optional(Box::new(Integer))),
            Optional(Box::new(Integer))
        );
        assert_eq!(Shape::factor(Any, Optional(Box::new(Integer))), Any);
        assert_eq!(
            Shape::factor(Optional(Box::new(Integer)), Optional(Box::new(Float))),
            Optional(Box::new(Float))
        );
        assert_eq!(
            Shape::factor(
                Optional(Box::new(Shape::String)),
                Optional(Box::new(Integer))
            ),
            Any
        );
    }

    #[test]
    fn fields() {
        let left = {
            let mut map = HashMap::new();
            map.insert("a".into(), Shape::Integer);
            map.insert("b".into(), Shape::Bool);
            map.insert("c".into(), Shape::Integer);
            map.insert("d".into(), Shape::String);
            map
        };
        let right = {
            let mut map = HashMap::new();
            map.insert("a".into(), Shape::Integer);
            map.insert("c".into(), Shape::Float);
            map.insert("d".into(), Shape::Null);
            map.insert("e".into(), Shape::Any);
            map
        };

        let res = {
            let mut v = Shape::factor_fields(left, right)
                .into_iter()
                .collect::<Vec<_>>();
            v.sort_by(|(l, _), (r, _)| l.cmp(&r));
            v
        };

        assert_eq!(
            res,
            vec![
                ("a".into(), Shape::Integer),
                ("b".into(), Shape::Optional(Box::new(Shape::Bool))),
                ("c".into(), Shape::Float),
                ("d".into(), Shape::Optional(Box::new(Shape::String))),
                ("e".into(), Shape::Any),
            ]
        )
    }
}
