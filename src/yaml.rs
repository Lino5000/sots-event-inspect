use std::{
    collections::BTreeMap,
    error::Error,
    fmt::Display,
    num::TryFromIntError,
};

#[derive(Debug)]
pub struct YamlError(String);

impl YamlError {
    fn new(msg: &str) -> Self {
        Self(msg.to_owned())
    }
}

impl Display for YamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for YamlError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl From<&str> for YamlError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for YamlError {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

macro_rules! impl_yamlerror_from_error {
    ($t:ty) => {
        impl From<$t> for YamlError {
            fn from(value: $t) -> Self {
                Self::new(&value.to_string())
            }
        }
    };
}
impl_yamlerror_from_error!(TryFromIntError);

#[derive(Debug)]
pub enum Field {
    Struct(BTreeMap<String, Field>),
    List(Vec<Field>),
    Bool(bool),
    Uint(u64),
    Int(i64),
    Float(f64),
    Null,
    Str(String),
}

struct FieldVisitor;

impl<'de> serde::de::Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid Unity yaml field")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut map_struct = BTreeMap::new();
        while let Some((key, value)) = map.next_entry()? {
            map_struct.insert(key, value);
        }
        Ok(Field::Struct(map_struct))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, 
    {
        let mut vec_list = Vec::new();
        while let Some(value) = seq.next_element::<Field>()? {
            vec_list.push(value);
        }
        Ok(Field::List(vec_list))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Bool(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Float(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Null)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Uint(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Int(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Str(v.to_owned()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Str(v.to_owned()))
    }
}

impl<'de> serde::Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>
    {
        deserializer.deserialize_any(FieldVisitor)
    }
}

#[macro_export]
macro_rules! field_value_type {
    (Uint) => { u64 };
    (Struct) => { std::collections::BTreeMap<String, Field> };
    (List) => { Vec<Field> };
    (Bool) => { bool };
    (Uint) => { u64 };
    (Int) => { i64 };
    (Float) => { f64 };
    (Str) => { String };
}

#[macro_export]
macro_rules! field_get_body {
    ($var:ident, $t:tt, $key:tt) => {
            constrain_field_get_body::<_, field_value_type!($t)>(|event_id, map| {
                let key: &str = stringify!($key);
                let Some(field) = map.get(key.clone()) else {
                    println!("{:?}", map);
                    return Err(format!("{}Field didn't contain `{}` key.",
                               if let Some(id) = event_id {
                                   format!("event {}: ", id)
                               } else { "".to_owned() },
                               key).into());
                };
                let Field::$t($var) = field else {
                    println!("{:?}", field);
                    return Err(format!("{}Field entry `{}` is not of type {}.",
                               if let Some(id) = event_id {
                                   format!("event {}: ", id)
                               } else { "".to_owned() },
                               key,
                               stringify!($t)).into());
                };
                Ok($var)
            })
    };
}

#[macro_export]
macro_rules! field_get {
    (let $var:ident: $t:tt = $map:ident.$key:tt) => {
        let $var = {
            let get = field_get_body!($var, $t, $key);
            get(None, $map)
        }?;
    };
    (event $id:expr, let $var:ident: $t:tt = $map:ident.$key:tt) => {
        let $var = {
            let get = field_get_body!($var, $t, $key);
            get(Some($id), $map)
        }?;
    };
}

pub fn constrain_field_get_body<F, R>(f: F) -> F
    where
        F: for<'a> Fn(Option<&String>, &'a std::collections::BTreeMap<String, Field>) -> Result<&'a R, YamlError>
{
    f
}

