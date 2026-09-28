//! Serialize `ProgressiveVariableList<ProgressiveVariableList<u8>>` as a list of 0x-prefixed hex
//! strings.
//!
//! The progressive (EIP-7688) counterpart of [`list_of_hex_var_list`](super::list_of_hex_var_list).
use crate::ProgressiveVariableList;
use serde::{de::Error, ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;
use typenum::Unsigned;

/// The inner byte list. `M` is its optional length limit.
pub struct WrappedListOwned<M>(ProgressiveVariableList<u8, M>);

impl<'de, M> Deserialize<'de> for WrappedListOwned<M>
where
    M: Unsigned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(super::hex_prog_var_list::deserialize(deserializer)?))
    }
}

pub struct WrappedListRef<'a, M>(&'a ProgressiveVariableList<u8, M>);

impl<M> Serialize for WrappedListRef<'_, M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::hex_prog_var_list::serialize(self.0, serializer)
    }
}

pub fn serialize<S, M, N>(
    list: &ProgressiveVariableList<ProgressiveVariableList<u8, M>, N>,
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

pub struct Visitor<M, N> {
    _phantom_m: PhantomData<M>,
    _phantom_n: PhantomData<N>,
}

impl<M, N> Default for Visitor<M, N> {
    fn default() -> Self {
        Self {
            _phantom_m: PhantomData,
            _phantom_n: PhantomData,
        }
    }
}

impl<'a, M, N> serde::de::Visitor<'a> for Visitor<M, N>
where
    M: Unsigned,
    N: Unsigned,
{
    type Value = ProgressiveVariableList<ProgressiveVariableList<u8, M>, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list of 0x-prefixed hex strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut list = ProgressiveVariableList::empty();
        while let Some(val) = seq.next_element::<WrappedListOwned<M>>()? {
            list.push(val.0).map_err(A::Error::custom)?;
        }
        Ok(list)
    }
}

pub fn deserialize<'de, D, M, N>(
    deserializer: D,
) -> Result<ProgressiveVariableList<ProgressiveVariableList<u8, M>, N>, D::Error>
where
    D: Deserializer<'de>,
    M: Unsigned,
    N: Unsigned,
{
    deserializer.deserialize_seq(Visitor::default())
}

#[cfg(test)]
mod test {
    use crate::ProgressiveVariableList;
    use serde_derive::{Deserialize, Serialize};
    use typenum::U2;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Obj {
        #[serde(with = "crate::serde_utils::prog_list_of_hex_prog_var_list")]
        lists: ProgressiveVariableList<ProgressiveVariableList<u8>>,
    }

    #[test]
    fn round_trip_hex() {
        let obj = Obj {
            lists: ProgressiveVariableList::new(vec![
                ProgressiveVariableList::new(vec![1, 2, 3]).unwrap(),
                ProgressiveVariableList::new(vec![255]).unwrap(),
            ])
            .unwrap(),
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

    #[derive(Debug, Deserialize)]
    struct Bounded {
        #[serde(with = "crate::serde_utils::prog_list_of_hex_prog_var_list")]
        lists: ProgressiveVariableList<ProgressiveVariableList<u8>, U2>,
    }

    #[test]
    fn accepts_list_at_limit() {
        let json = r#"{"lists":["0x01","0x02"]}"#;
        assert_eq!(
            serde_json::from_str::<Bounded>(json).unwrap().lists.len(),
            2
        );
    }

    #[test]
    fn fails_at_first_item_past_limit() {
        // The item after the limit is not hex. Only an early check reports the limit.
        let json = r#"{"lists":["0x01","0x02","0x03","not hex"]}"#;
        let err = serde_json::from_str::<Bounded>(json).unwrap_err();
        assert!(
            err.to_string()
                .contains("Index out of bounds: index 3, length 2"),
            "{err}"
        );
    }
}
