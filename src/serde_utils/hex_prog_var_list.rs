//! Serialize `ProgressiveVariableList<u8>` as a 0x-prefixed hex string.
//!
//! The progressive (EIP-7688) counterpart of [`hex_var_list`](super::hex_var_list).
use crate::ProgressiveVariableList;
use serde::{de::Error, Deserializer, Serializer};
use serde_utils::hex::{self, PrefixedHexVisitor};
use typenum::Unsigned;

pub fn serialize<S, N>(
    bytes: &ProgressiveVariableList<u8, N>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(&**bytes))
}

pub fn deserialize<'de, D, N>(deserializer: D) -> Result<ProgressiveVariableList<u8, N>, D::Error>
where
    D: Deserializer<'de>,
    N: Unsigned,
{
    let bytes = deserializer.deserialize_str(PrefixedHexVisitor)?;
    ProgressiveVariableList::new(bytes).map_err(D::Error::custom)
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
            bytes: ProgressiveVariableList::new(vec![1, 2, 3, 255]).unwrap(),
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

    #[derive(Debug, PartialEq, Deserialize)]
    struct Bounded {
        #[serde(with = "crate::serde_utils::hex_prog_var_list")]
        bytes: ProgressiveVariableList<u8, typenum::U4>,
    }

    #[test]
    fn limit_rejects_oversized_hex() {
        assert!(serde_json::from_str::<Bounded>(r#"{"bytes":"0x01020304"}"#).is_ok());
        assert!(serde_json::from_str::<Bounded>(r#"{"bytes":"0x0102030405"}"#).is_err());
    }
}
