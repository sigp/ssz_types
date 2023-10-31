use crate::tree_hash::optional_tree_hash_root;
use derivative::Derivative;
use serde_derive::{Deserialize, Serialize};
use tree_hash::Hash256;

pub use typenum;

/// Emulates a SSZ `Optional` (distinct from a Rust `Option`).
///
/// This SSZ type is defined in EIP-6475.
///
/// This struct is backed by a Rust `Option` and its behaviour is defined by the variant.
///
/// If `Some`, it will serialize with a 1-byte identifying prefix with a value of 1 followed by the
/// serialized internal type.
/// If `None`, it will serialize as `null`.
///
/// `Optional` will Merklize in the following ways:
/// `if None`: Merklize as an empty `VariableList`
/// `if Some(T)`: Merklize as a `VariableList` of length 1 whose single value is `T`.
///
/// ## Example
///
/// ```
/// use ssz_types::{Optional, typenum::*, VariableList};
/// use tree_hash::TreeHash;
/// use ssz::Encode;
///
/// // Create an `Optional` from an `Option` that is `Some`.
/// let some: Option<u8> = Some(9);
/// let ssz: Optional<u8> = Optional::from(some);
/// let serialized: &[u8] = &ssz.as_ssz_bytes();
/// assert_eq!(serialized, &[1, 9]);
///
/// let root = ssz.tree_hash_root();
/// let equivalent_list: VariableList<u64, U1> = VariableList::from(vec![9; 1]);
/// assert_eq!(root, equivalent_list.tree_hash_root());
///
/// // Create an `Optional` from an `Option` that is `None`.
/// let none: Option<u8> = None;
/// let ssz: Optional<u8> = Optional::from(none);
/// let serialized: &[u8] = &ssz.as_ssz_bytes();
/// let null: &[u8] = &[];
/// assert_eq!(serialized, null);
///
/// let root = ssz.tree_hash_root();
/// let equivalent_list: VariableList<u8, U0> = VariableList::from(vec![]);
/// assert_eq!(root, equivalent_list.tree_hash_root());
///
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(PartialEq, Hash(bound = "T: std::hash::Hash"))]
#[serde(transparent)]
pub struct Optional<T> {
    optional: Option<T>,
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(optional: Option<T>) -> Self {
        Self { optional }
    }
}

impl<T> From<Optional<T>> for Option<T> {
    fn from(val: Optional<T>) -> Option<T> {
        val.optional
    }
}

impl<T> Default for Optional<T> {
    fn default() -> Self {
        Self { optional: None }
    }
}

impl<T> tree_hash::TreeHash for Optional<T>
where
    T: tree_hash::TreeHash,
{
    fn tree_hash_type() -> tree_hash::TreeHashType {
        tree_hash::TreeHashType::List
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_root(&self) -> Hash256 {
        let root = optional_tree_hash_root::<T>(&self.optional);

        let length = match &self.optional {
            None => 0,
            Some(_) => 1,
        };

        tree_hash::mix_in_length(&root, length)
    }
}

impl<T> ssz::Encode for Optional<T>
where
    T: ssz::Encode,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_bytes_len(&self) -> usize {
        match &self.optional {
            None => 0,
            Some(val) => val.ssz_bytes_len() + 1,
        }
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        match &self.optional {
            None => (),
            Some(val) => {
                let mut optional_identifier = vec![1];
                buf.append(&mut optional_identifier);
                val.ssz_append(buf)
            }
        }
    }
}

impl<T> ssz::Decode for Optional<T>
where
    T: ssz::Decode,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if let Some((first, rest)) = bytes.split_first() {
            if first == &0x01 {
                return Ok(Optional {
                    optional: Some(T::from_ssz_bytes(&rest)?),
                });
            } else {
                // An `Optional` must always contains `0x01` as the first byte.
                // Might be worth having an explicit error variant in ssz::DecodeError.
                return Err(ssz::DecodeError::BytesInvalid(
                    "Missing Optional identifier byte".to_string(),
                ));
            }
        } else {
            Ok(Optional { optional: None })
        }
    }
}

/// TODO Use a more robust `Arbitrary` impl.
#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Optional<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let option = Some(<T>::arbitrary(u).unwrap());
        Ok(Self::from(option))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{FixedVector, VariableList};
    use ssz::*;
    use ssz_derive::{Decode, Encode};
    use tree_hash::TreeHash;
    use tree_hash_derive::TreeHash;
    use typenum::*;

    #[test]
    fn encode() {
        let some: Optional<u8> = Some(42).into();
        let bytes: Vec<u8> = vec![1, 42];
        assert_eq!(some.as_ssz_bytes(), bytes);

        let none: Optional<u8> = None.into();
        let empty: Vec<u8> = vec![];
        assert_eq!(none.as_ssz_bytes(), empty);
    }

    #[test]
    fn decode() {
        let bytes = &[1, 42, 0, 0, 0, 0, 0, 0, 0];
        let some: Optional<u64> = Optional::from_ssz_bytes(bytes).unwrap();
        assert_eq!(Some(42), some.optional);

        let empty = &[];
        let none: Optional<u64> = Optional::from_ssz_bytes(empty).unwrap();
        assert_eq!(None, none.optional);
    }

    #[test]
    fn tree_hash_none() {
        // None should merklize the same as an empty VariableList.
        let none: Optional<u8> = Optional::from(None);
        let empty_list: VariableList<u8, U0> = VariableList::from(vec![]);
        assert_eq!(none.tree_hash_root(), empty_list.tree_hash_root());
    }

    #[test]
    fn tree_hash_some_int() {
        // Optional should merklize the same as a length 1 VariableList.
        let some_int: Optional<u8> = Optional::from(Some(9));
        let list_int: VariableList<u8, U1> = VariableList::from(vec![9; 1]);
        assert_eq!(some_int.tree_hash_root(), list_int.tree_hash_root());
    }

    #[test]
    fn tree_hash_some_list() {
        // Optional should merklize the same as a length 1 VariableList.
        let list: VariableList<u8, U16> = VariableList::from(vec![9; 16]);
        let some_list: Optional<VariableList<u8, U16>> = Optional::from(Some(list.clone()));
        let list_list: VariableList<VariableList<u8, U16>, U1> = VariableList::from(vec![list; 1]);
        assert_eq!(some_list.tree_hash_root(), list_list.tree_hash_root());
    }

    #[test]
    fn tree_hash_some_vec() {
        // Optional should merklize the same as a length 1 VariableList.
        let vec: FixedVector<u8, U16> = FixedVector::from(vec![9; 16]);
        let some_vec: Optional<FixedVector<u8, U16>> = Optional::from(Some(vec.clone()));
        let list_vec: VariableList<FixedVector<u8, U16>, U1> = VariableList::from(vec![vec; 1]);
        assert_eq!(some_vec.tree_hash_root(), list_vec.tree_hash_root());
    }

    #[test]
    fn tree_hash_some_object() {
        #[derive(TreeHash, Decode, Encode)]
        struct Object {
            a: u8,
            b: u8,
        }

        // Optional should merklize the same as a length 1 VariableList. Note the 1-byte identifier
        // during deserialization.
        let optional_object: Optional<Object> = Optional::from_ssz_bytes(&[1, 11, 9]).unwrap();
        let list_object: VariableList<Object, U1> = VariableList::from_ssz_bytes(&[11, 9]).unwrap();

        assert_eq!(
            optional_object.tree_hash_root(),
            list_object.tree_hash_root()
        );
    }
}
