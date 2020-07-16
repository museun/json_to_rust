pub type HashMap<K, V> = indexmap::IndexMap<K, V>;
pub type Map = HashMap<String, Shape>;

mod local;
mod shape;

pub use local::Local;
pub use shape::Shape;
