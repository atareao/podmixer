use serde::{Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum Data {
    None,
    One(Value),
    #[allow(unused)]
    Some(Vec<Value>),
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Data::None => serializer.serialize_none(),
            Data::One(value) => serializer.serialize_some(value),
            Data::Some(values) => serializer.serialize_some(values),
        }
    }
}


