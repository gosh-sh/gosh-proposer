use serde::{Deserializer, Deserialize};

pub mod tracing;

pub fn deserialize_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(u128::from_str_radix(&s, 10).unwrap_or_default())
}
