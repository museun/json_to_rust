use super::{HashMap, Map};
use json::JsonValue as Value;

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
            Value::Null => Self::Null,
            Value::Boolean(..) => Self::Bool,
            Value::Number(..) => {
                if val.as_i64().is_some() {
                    Self::Integer
                } else {
                    Self::Float
                }
            }

            Value::String(..) | Value::Short(..) => Self::String,
            Value::Array(ref array) => {
                let len = array.len();
                if len > 1 && len <= max_tuple {
                    Self::Tuple(
                        array.iter().map(|s| Self::new(s, max_tuple)).collect(),
                        len as _,
                    )
                } else {
                    let ty = array.iter().fold(Self::Bottom, |left, v| {
                        Self::factor(left, Self::new(v, max_tuple))
                    });
                    Self::Array(Box::new(ty))
                }
            }
            Value::Object(ref map) => {
                let fields = map
                    .iter()
                    .map(|(k, v)| (k.to_string(), Self::new(v, max_tuple)));
                Self::Object(fields.collect())
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn root(&self) -> &'static str {
        match self {
            Self::Bottom => "Bottom",
            Self::Any => "Any",
            Self::Optional(_) => "Optional",
            Self::Null => "Null",
            Self::Bool => "Bool",
            Self::String => "String",
            Self::Integer => "Integer",
            Self::Float => "Float",
            Self::Array(_) => "Array",
            Self::Object(_) => "Object",
            Self::Tuple(_, _) => "Tuple",
            Self::Map(_) => "Map",
            Self::Opaque(_) => "Opaque",
        }
    }

    pub(crate) fn fold(shapes: impl IntoIterator<Item = Self>) -> Self {
        shapes.into_iter().fold(Self::Bottom, Self::factor)
    }

    pub(crate) fn factor(left: Self, right: Self) -> Self {
        eprintln!("{} | {}", left.root(), right.root());

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
            (Self::Object(left), Self::Object(right)) => Self::factor_fields(left, right),

            // equal opaque types
            (Self::Opaque(name), ..) | (.., Self::Opaque(name)) => Self::Opaque(name),

            // otherwise anything
            _ => Self::Any,
        }
    }

    fn factor_fields(left: Map, mut right: Map) -> Self {
        // if the lengths are different we shouldn't unify
        // so we need to track the minimum length we've seen so far

        // match (left.keys().len(), right.keys().len()) {
        //     (_, 0) => return Self::Object(left),
        //     (0, _) => return Self::Object(right),
        //     (l, r) if l != r => {
        //         let tup = vec![Self::Object(left), Self::Object(right)];
        //         return Self::Tuple(tup, 2);
        //     }
        //     _ => {}
        // }

        if left == right {
            return Self::Object(left);
        }

        let mut unified: HashMap<_, _> = left
            .into_iter()
            .map(|(k, v)| {
                let v = match right.remove(&k) {
                    Some(r) => Self::factor(v, r),
                    None => v.into_optional(),
                };
                (k, v)
            })
            .collect();

        unified.extend(right.into_iter().map(|(k, v)| (k, v.into_optional())));
        Self::Object(unified)
    }

    fn into_optional(self) -> Self {
        match self {
            Self::Bottom | Self::Any | Self::Null | Self::Optional(_) => self,
            other => Self::Optional(Box::new(other)),
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

    // TODO moved factor_fields up
    // #[test]
    // fn fields() {
    //     let left = {
    //         let mut map = HashMap::new();
    //         map.insert("a".into(), Shape::Integer);
    //         map.insert("b".into(), Shape::Bool);
    //         map.insert("c".into(), Shape::Integer);
    //         map.insert("d".into(), Shape::String);
    //         map
    //     };
    //     let right = {
    //         let mut map = HashMap::new();
    //         map.insert("a".into(), Shape::Integer);
    //         map.insert("c".into(), Shape::Float);
    //         map.insert("d".into(), Shape::Null);
    //         map.insert("e".into(), Shape::Any);
    //         map
    //     };

    //     let res = {
    //         let mut v = Shape::factor_fields(left, right)
    //             .into_iter()
    //             .collect::<Vec<_>>();
    //         v.sort_by(|(l, _), (r, _)| l.cmp(&r));
    //         v
    //     };

    //     assert_eq!(
    //         res,
    //         vec![
    //             ("a".into(), Shape::Integer),
    //             ("b".into(), Shape::Optional(Box::new(Shape::Bool))),
    //             ("c".into(), Shape::Float),
    //             ("d".into(), Shape::Optional(Box::new(Shape::String))),
    //             ("e".into(), Shape::Any),
    //         ]
    //     )
    // }
}
