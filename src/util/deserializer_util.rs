use serde::Deserialize;
 
pub fn trim<'de, D>(deserializer: D) -> Result<String, D::Error> where D: serde::Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.trim().to_string())
}

pub fn trim_lower<'de, D>(deserializer: D) -> Result<String, D::Error> where D: serde::Deserializer<'de> {
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.trim().to_lowercase())
}

