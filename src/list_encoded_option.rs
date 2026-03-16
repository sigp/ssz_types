use crate::tree_hash::vec_tree_hash_root;
use crate::VariableList;
use serde_derive::{Deserialize, Serialize};
use smallvec::SmallVec;
use ssz::{Decode, DecodeError, Encode};
use std::ops::{Deref, DerefMut};
use tree_hash::{Hash256, PackedEncoding, TreeHash, TreeHashType};
use typenum::U1;

/// An `Option<T>` that is SSZ-encoded as a `VariableList<T, 1>`.
///
/// `None` is encoded as an empty list and `Some(value)` as a single-element list.
/// This is useful for representing optional fields in SSZ containers without
/// relying on a union type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ListEncodedOption<T>(pub Option<T>);

impl<T> Deref for ListEncodedOption<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ListEncodedOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Option<T>> for ListEncodedOption<T> {
    fn from(option: Option<T>) -> Self {
        Self(option)
    }
}

impl<T> From<ListEncodedOption<T>> for Option<T> {
    fn from(option: ListEncodedOption<T>) -> Self {
        option.0
    }
}

impl<T: Encode> Encode for ListEncodedOption<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        if let Some(t) = &self.0 {
            SmallVec::from_buf([t]).ssz_append(buf)
        }
    }

    fn ssz_bytes_len(&self) -> usize {
        if let Some(t) = &self.0 {
            SmallVec::from_buf([t]).ssz_bytes_len()
        } else {
            0
        }
    }
}

impl<T: Decode + 'static> Decode for ListEncodedOption<T> {
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let list = VariableList::<T, U1>::from_ssz_bytes(bytes)?;
        Ok(Self(list.into_iter().next()))
    }
}

impl<T: TreeHash> TreeHash for ListEncodedOption<T> {
    fn tree_hash_type() -> TreeHashType {
        TreeHashType::List
    }

    fn tree_hash_packed_encoding(&self) -> PackedEncoding {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("List should never be packed.")
    }

    fn tree_hash_root(&self) -> Hash256 {
        let slice: &[T] = match &self.0 {
            Some(val) => std::slice::from_ref(val),
            None => &[],
        };
        let root = vec_tree_hash_root::<T>(slice, 1);
        tree_hash::mix_in_length(&root, slice.len())
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for ListEncodedOption<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self(Option::arbitrary(u)?))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ssz::{Decode, Encode};
    use typenum::U1;

    #[test]
    fn none_encodes_as_empty_list() {
        let option: ListEncodedOption<u64> = None.into();
        let list: VariableList<u64, U1> = VariableList::try_from(vec![]).unwrap();
        assert_eq!(option.as_ssz_bytes(), list.as_ssz_bytes());
        assert_eq!(option.ssz_bytes_len(), list.ssz_bytes_len());
    }

    #[test]
    fn some_encodes_as_single_element_list() {
        let option: ListEncodedOption<u64> = Some(42u64).into();
        let list: VariableList<u64, U1> = VariableList::try_from(vec![42u64]).unwrap();
        assert_eq!(option.as_ssz_bytes(), list.as_ssz_bytes());
        assert_eq!(option.ssz_bytes_len(), list.ssz_bytes_len());
    }

    #[test]
    fn round_trip_none() {
        let original: ListEncodedOption<u64> = None.into();
        let bytes = original.as_ssz_bytes();
        let decoded = ListEncodedOption::<u64>::from_ssz_bytes(&bytes).unwrap();
        assert!(decoded.is_none());
    }

    #[test]
    fn round_trip_some() {
        let original: ListEncodedOption<u64> = Some(123u64).into();
        let bytes = original.as_ssz_bytes();
        let decoded = ListEncodedOption::<u64>::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(*decoded, Some(123u64));
    }

    #[test]
    fn none_variable_encodes_as_empty_list() {
        let option: ListEncodedOption<Vec<u8>> = None.into();
        let list: VariableList<Vec<u8>, U1> = VariableList::try_from(vec![]).unwrap();
        assert_eq!(option.as_ssz_bytes(), list.as_ssz_bytes());
        assert_eq!(option.ssz_bytes_len(), list.ssz_bytes_len());
    }

    #[test]
    fn some_variable_encodes_as_single_element_list() {
        let value = vec![1u8, 2, 3];
        let option: ListEncodedOption<Vec<u8>> = Some(value.clone()).into();
        let list: VariableList<Vec<u8>, U1> = VariableList::try_from(vec![value]).unwrap();
        assert_eq!(option.as_ssz_bytes(), list.as_ssz_bytes());
        assert_eq!(option.ssz_bytes_len(), list.ssz_bytes_len());
    }

    #[test]
    fn round_trip_variable_none() {
        let original: ListEncodedOption<Vec<u8>> = None.into();
        let bytes = original.as_ssz_bytes();
        let decoded = ListEncodedOption::<Vec<u8>>::from_ssz_bytes(&bytes).unwrap();
        assert!(decoded.is_none());
    }

    #[test]
    fn round_trip_variable_some() {
        let original: ListEncodedOption<Vec<u8>> = Some(vec![4u8, 5, 6]).into();
        let bytes = original.as_ssz_bytes();
        let decoded = ListEncodedOption::<Vec<u8>>::from_ssz_bytes(&bytes).unwrap();
        assert_eq!(*decoded, Some(vec![4u8, 5, 6]));
    }

    #[test]
    fn rejects_two_element_list() {
        use typenum::U2;
        let list: VariableList<u64, U2> = VariableList::try_from(vec![1u64, 2]).unwrap();
        assert!(ListEncodedOption::<u64>::from_ssz_bytes(&list.as_ssz_bytes()).is_err());
    }

    #[test]
    fn is_variable_len() {
        assert!(!<ListEncodedOption<u64> as Encode>::is_ssz_fixed_len());
        assert!(!<ListEncodedOption<u64> as Decode>::is_ssz_fixed_len());
    }

    #[test]
    fn tree_hash_none_matches_empty_list() {
        let option: ListEncodedOption<u64> = None.into();
        let list: VariableList<u64, U1> = VariableList::try_from(vec![]).unwrap();
        assert_eq!(option.tree_hash_root(), list.tree_hash_root());
    }

    #[test]
    fn tree_hash_some_matches_single_element_list() {
        let option: ListEncodedOption<u64> = Some(42u64).into();
        let list: VariableList<u64, U1> = VariableList::try_from(vec![42u64]).unwrap();
        assert_eq!(option.tree_hash_root(), list.tree_hash_root());
    }

    #[test]
    fn tree_hash_variable_none_matches_empty_list() {
        let option: ListEncodedOption<VariableList<u8, typenum::U256>> = None.into();
        let list: VariableList<VariableList<u8, typenum::U256>, U1> =
            VariableList::try_from(vec![]).unwrap();
        assert_eq!(option.tree_hash_root(), list.tree_hash_root());
    }

    #[test]
    fn tree_hash_variable_some_matches_single_element_list() {
        let value: VariableList<u8, typenum::U256> =
            VariableList::try_from(vec![1u8, 2, 3]).unwrap();
        let option: ListEncodedOption<VariableList<u8, typenum::U256>> = Some(value.clone()).into();
        let list: VariableList<VariableList<u8, typenum::U256>, U1> =
            VariableList::try_from(vec![value]).unwrap();
        assert_eq!(option.tree_hash_root(), list.tree_hash_root());
    }
}
