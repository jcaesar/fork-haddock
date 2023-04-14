use std::{
    fmt::{self, Formatter},
    marker::PhantomData,
};

use console::{style, StyledObject};
use once_cell::sync::Lazy;
pub(crate) use regex;
use serde::{
    de::{self, Visitor},
    Deserializer, Serialize, Serializer,
};
use serde_with::{DeserializeAs, SerializeAs};

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

pub(crate) static STYLED_WARNING: Lazy<StyledObject<&str>> =
    Lazy::new(|| style("Warning:").yellow().bold());

pub(crate) struct DisplayFromAny;

impl<'de, T> DeserializeAs<'de, T> for DisplayFromAny
where
    T: From<String>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AnyVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for AnyVisitor<T>
        where
            T: From<String>,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                formatter.write_str("a displayable type")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(T::from(v.to_string()))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(T::from(v.to_string()))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(T::from(v.to_string()))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(T::from(v.to_string()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(T::from(v.to_string()))
            }
        }

        deserializer.deserialize_any(AnyVisitor(PhantomData))
    }
}

impl<T> SerializeAs<T> for DisplayFromAny
where
    T: Serialize,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        source.serialize(serializer)
    }
}
