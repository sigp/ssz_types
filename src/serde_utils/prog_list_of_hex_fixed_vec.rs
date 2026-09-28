//! Serialize `ProgressiveVariableList<FixedVector<u8, M>>` as list of 0x-prefixed hex string.
//!
//! The progressive (EIP-7688) counterpart of [`list_of_hex_fixed_vec`](super::list_of_hex_fixed_vec).
use crate::{FixedVector, ProgressiveVariableList};
use serde::{de::Error, ser::SerializeSeq, Deserializer, Serializer};
use std::marker::PhantomData;
use typenum::Unsigned;

// The element wrappers are identical to the bounded `list_of_hex_fixed_vec`, so reuse them.
pub use super::list_of_hex_fixed_vec::{WrappedListOwned, WrappedListRef};

pub fn serialize<S, M, N>(
    list: &ProgressiveVariableList<FixedVector<u8, M>, N>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    M: Unsigned,
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
    type Value = ProgressiveVariableList<FixedVector<u8, M>, N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list of 0x-prefixed hex bytes")
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
) -> Result<ProgressiveVariableList<FixedVector<u8, M>, N>, D::Error>
where
    D: Deserializer<'de>,
    M: Unsigned,
    N: Unsigned,
{
    deserializer.deserialize_seq(Visitor::default())
}

#[cfg(test)]
mod test {
    use crate::{FixedVector, ProgressiveVariableList};
    use serde_derive::Deserialize;
    use typenum::{U1, U2};

    #[derive(Debug, Deserialize)]
    struct Bounded {
        #[serde(with = "crate::serde_utils::prog_list_of_hex_fixed_vec")]
        vecs: ProgressiveVariableList<FixedVector<u8, U1>, U2>,
    }

    #[test]
    fn accepts_list_at_limit() {
        let json = r#"{"vecs":["0x01","0x02"]}"#;
        assert_eq!(serde_json::from_str::<Bounded>(json).unwrap().vecs.len(), 2);
    }

    #[test]
    fn fails_at_first_item_past_limit() {
        // The item after the limit is not hex. Only an early check reports the limit.
        let json = r#"{"vecs":["0x01","0x02","0x03","not hex"]}"#;
        let err = serde_json::from_str::<Bounded>(json).unwrap_err();
        assert!(
            err.to_string()
                .contains("Index out of bounds: index 3, length 2"),
            "{err}"
        );
    }
}
