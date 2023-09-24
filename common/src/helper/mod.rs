use serde::{Deserialize, Deserializer};

pub mod tracing;

pub fn deserialize_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse::<u128>().unwrap_or_default())
}
