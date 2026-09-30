//! Fixtures from ssz-specs.

use serde_derive::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub type_name: String,
    pub value: Value,
    pub serialized: String,
    pub root: String,
    pub raw_bytes: Option<String>,
    pub rejection_reason: Option<String>,
    #[serde(rename = "_info")]
    pub info: Info,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub fixture_format: String,
}

pub fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    let digits = text
        .strip_prefix("0x")
        .ok_or_else(|| format!("hex data must start with 0x: {text}"))?;
    hex::decode(digits).map_err(|e| format!("invalid hex data: {e}"))
}
