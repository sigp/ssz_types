//! Serialize `ProgressiveVariableList<FixedVector<u8, M>>` as list of 0x-prefixed hex string.
//!
//! The progressive (EIP-7688) counterpart of [`list_of_hex_fixed_vec`](super::list_of_hex_fixed_vec).
use crate::{FixedVector, ProgressiveVariableList};
use serde::{ser::SerializeSeq, Deserializer, Serializer};
use std::marker::PhantomData;
use typenum::Unsigned;

// The element wrappers are identical to the bounded `list_of_hex_fixed_vec`, so reuse them.
pub use super::list_of_hex_fixed_vec::{WrappedListOwned, WrappedListRef};

pub fn serialize<S, M>(
    list: &ProgressiveVariableList<FixedVector<u8, M>>,
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

#[derive(Default)]
pub struct Visitor<M> {
    _phantom_m: PhantomData<M>,
}

impl<'a, M> serde::de::Visitor<'a> for Visitor<M>
where
    M: Unsigned,
{
    type Value = ProgressiveVariableList<FixedVector<u8, M>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a list of 0x-prefixed hex bytes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut list = ProgressiveVariableList::empty();

        while let Some(val) = seq.next_element::<WrappedListOwned<M>>()? {
            list.push(val.0);
        }

        Ok(list)
    }
}

pub fn deserialize<'de, D, M>(
    deserializer: D,
) -> Result<ProgressiveVariableList<FixedVector<u8, M>>, D::Error>
where
    D: Deserializer<'de>,
    M: Unsigned,
{
    deserializer.deserialize_seq(Visitor::default())
}
