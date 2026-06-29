//! Serialize `ProgressiveVariableList<u8>` as a 0x-prefixed hex string.
//!
//! The progressive (EIP-7688) counterpart of [`hex_var_list`](super::hex_var_list).
use crate::ProgressiveVariableList;
use serde::{Deserializer, Serializer};
use serde_utils::hex::{self, PrefixedHexVisitor};

pub fn serialize<S>(bytes: &ProgressiveVariableList<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(&**bytes))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<ProgressiveVariableList<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = deserializer.deserialize_str(PrefixedHexVisitor)?;
    Ok(ProgressiveVariableList::new(bytes))
}

#[cfg(test)]
mod test {
    use crate::ProgressiveVariableList;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Obj {
        #[serde(with = "crate::serde_utils::hex_prog_var_list")]
        bytes: ProgressiveVariableList<u8>,
    }

    #[test]
    fn round_trip_hex() {
        let obj = Obj {
            bytes: ProgressiveVariableList::new(vec![1, 2, 3, 255]),
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(json, r#"{"bytes":"0x010203ff"}"#);
        assert_eq!(serde_json::from_str::<Obj>(&json).unwrap(), obj);
    }

    #[test]
    fn empty() {
        let obj = Obj {
            bytes: ProgressiveVariableList::empty(),
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(json, r#"{"bytes":"0x"}"#);
        assert_eq!(serde_json::from_str::<Obj>(&json).unwrap(), obj);
    }
}
