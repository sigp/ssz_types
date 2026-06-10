//! Serialize `ProgressiveVariableList<ProgressiveVariableList<u8>>` as a list of 0x-prefixed hex
//! strings.
//!
//! The progressive (EIP-7688) counterpart of [`list_of_hex_var_list`](super::list_of_hex_var_list).
use crate::ProgressiveVariableList;
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize)]
#[serde(transparent)]
pub struct WrappedListOwned(
    #[serde(with = "crate::serde_utils::hex_prog_var_list")] ProgressiveVariableList<u8>,
);

#[derive(Serialize)]
#[serde(transparent)]
pub struct WrappedListRef<'a>(
    #[serde(with = "crate::serde_utils::hex_prog_var_list")] &'a ProgressiveVariableList<u8>,
);

pub fn serialize<S>(
    list: &ProgressiveVariableList<ProgressiveVariableList<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(list.len()))?;
    for bytes in list {
        seq.serialize_element(&WrappedListRef(bytes))?;
    }
    seq.end()
}

struct Visitor;

impl<'a> serde::de::Visitor<'a> for Visitor {
    type Value = ProgressiveVariableList<ProgressiveVariableList<u8>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list of 0x-prefixed hex strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut list = Vec::new();
        while let Some(val) = seq.next_element::<WrappedListOwned>()? {
            list.push(val.0);
        }
        Ok(ProgressiveVariableList::new(list))
    }
}

pub fn deserialize<'de, D>(
    deserializer: D,
) -> Result<ProgressiveVariableList<ProgressiveVariableList<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(Visitor)
}

#[cfg(test)]
mod test {
    use crate::ProgressiveVariableList;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Obj {
        #[serde(with = "crate::serde_utils::list_of_hex_prog_var_list")]
        lists: ProgressiveVariableList<ProgressiveVariableList<u8>>,
    }

    #[test]
    fn round_trip_hex() {
        let obj = Obj {
            lists: ProgressiveVariableList::new(vec![
                ProgressiveVariableList::new(vec![1, 2, 3]),
                ProgressiveVariableList::new(vec![255]),
            ]),
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(json, r#"{"lists":["0x010203","0xff"]}"#);
        assert_eq!(serde_json::from_str::<Obj>(&json).unwrap(), obj);
    }

    #[test]
    fn empty() {
        let obj = Obj {
            lists: ProgressiveVariableList::empty(),
        };
        let json = serde_json::to_string(&obj).unwrap();
        assert_eq!(json, r#"{"lists":[]}"#);
        assert_eq!(serde_json::from_str::<Obj>(&json).unwrap(), obj);
    }
}
