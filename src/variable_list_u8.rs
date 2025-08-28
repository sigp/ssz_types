use crate::tree_hash::vec_tree_hash_root;
use crate::{Error, VariableList};
use serde::Deserialize;
use serde_derive::Serialize;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;
use tree_hash::Hash256;
use typenum::Unsigned;

pub use typenum;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct VariableListU8<N> {
    inner: VariableList<u8, N>,
}

impl<N> PartialEq for VariableListU8<N> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<N> Eq for VariableListU8<N> {}
impl<N> std::hash::Hash for VariableListU8<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<N: Unsigned> VariableListU8<N> {
    pub fn new(vec: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            inner: VariableList::new(vec)?,
        })
    }

    pub fn empty() -> Self {
        Self {
            inner: VariableList::empty(),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn max_len() -> usize {
        VariableList::<u8, N>::max_len()
    }

    pub fn push(&mut self, value: u8) -> Result<(), Error> {
        self.inner.push(value)
    }
}

impl<N: Unsigned> TryFrom<Vec<u8>> for VariableListU8<N> {
    type Error = Error;

    fn try_from(vec: Vec<u8>) -> Result<Self, Error> {
        Self::new(vec)
    }
}

impl<N: Unsigned> From<VariableListU8<N>> for Vec<u8> {
    fn from(list: VariableListU8<N>) -> Vec<u8> {
        list.inner.into()
    }
}

impl<N: Unsigned> Default for VariableListU8<N> {
    fn default() -> Self {
        Self {
            inner: VariableList::default(),
        }
    }
}

impl<N: Unsigned, I: SliceIndex<[u8]>> Index<I> for VariableListU8<N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.inner, index)
    }
}

impl<N: Unsigned, I: SliceIndex<[u8]>> IndexMut<I> for VariableListU8<N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.inner, index)
    }
}

impl<N: Unsigned> Deref for VariableListU8<N> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.inner
    }
}

impl<N: Unsigned> DerefMut for VariableListU8<N> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

impl<'a, N: Unsigned> IntoIterator for &'a VariableListU8<N> {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<N: Unsigned> IntoIterator for VariableListU8<N> {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<N: Unsigned> tree_hash::TreeHash for VariableListU8<N> {
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
        let root = vec_tree_hash_root::<u8>(&self.inner, N::to_usize());
        tree_hash::mix_in_length(&root, self.len())
    }
}

impl<N: Unsigned> ssz::Encode for VariableListU8<N> {
    fn is_ssz_fixed_len() -> bool {
        <Vec<u8>>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <Vec<u8>>::ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        self.inner.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.inner.ssz_append(buf)
    }
}

impl<N: Unsigned> ssz::TryFromIter<u8> for VariableListU8<N> {
    type Error = Error;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = u8>,
    {
        let inner = VariableList::try_from_iter(value)?;
        Ok(Self { inner })
    }
}

impl<N> ssz::Decode for VariableListU8<N>
where
    N: Unsigned,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let inner = VariableList::from_ssz_bytes(bytes)?;
        Ok(Self { inner })
    }
}

impl<'de, N> Deserialize<'de> for VariableListU8<N>
where
    N: Unsigned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = VariableList::<u8, N>::deserialize(deserializer)?;
        Ok(Self { inner })
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, N: 'static + Unsigned> arbitrary::Arbitrary<'a> for VariableListU8<N> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let inner = VariableList::<u8, N>::arbitrary(u)?;
        Ok(Self { inner })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ssz::*;
    use std::collections::HashSet;
    use tree_hash::{merkle_root, TreeHash};
    use typenum::*;

    #[test]
    fn new() {
        let vec = vec![42; 5];
        let fixed: Result<VariableListU8<U4>, _> = VariableListU8::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 3];
        let fixed: Result<VariableListU8<U4>, _> = VariableListU8::new(vec);
        assert!(fixed.is_ok());

        let vec = vec![42; 4];
        let fixed: Result<VariableListU8<U4>, _> = VariableListU8::new(vec);
        assert!(fixed.is_ok());
    }

    #[test]
    fn indexing() {
        let vec = vec![1, 2];

        let mut fixed: VariableListU8<U8192> = vec.clone().try_into().unwrap();

        assert_eq!(fixed[0], 1);
        assert_eq!(&fixed[0..1], &vec[0..1]);
        assert_eq!((fixed[..]).len(), 2);

        fixed[1] = 3;
        assert_eq!(fixed[1], 3);
    }

    #[test]
    fn length() {
        let vec = vec![42; 5];
        let err = VariableListU8::<U4>::try_from(vec.clone()).unwrap_err();
        assert_eq!(err, Error::OutOfBounds { i: 5, len: 4 });

        let vec = vec![42; 3];
        let fixed: VariableListU8<U4> = VariableListU8::try_from(vec.clone()).unwrap();
        assert_eq!(&fixed[0..3], &vec[..]);
        assert_eq!(&fixed[..], &vec![42, 42, 42][..]);

        let vec = vec![];
        let fixed: VariableListU8<U4> = VariableListU8::try_from(vec).unwrap();
        assert_eq!(&fixed[..], &[] as &[u8]);
    }

    #[test]
    fn deref() {
        let vec = vec![0, 2, 4, 6];
        let fixed: VariableListU8<U4> = VariableListU8::try_from(vec).unwrap();

        assert_eq!(fixed.first(), Some(&0));
        assert_eq!(fixed.get(3), Some(&6));
        assert_eq!(fixed.get(4), None);
    }

    #[test]
    fn encode() {
        let vec: VariableListU8<U2> = vec![0; 2].try_into().unwrap();
        assert_eq!(vec.as_ssz_bytes(), vec![0, 0]);
        assert_eq!(<VariableListU8<U2> as Encode>::ssz_fixed_len(), 4);
    }

    fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn u8_len_8() {
        round_trip::<VariableListU8<U8>>(vec![42; 8].try_into().unwrap());
        round_trip::<VariableListU8<U8>>(vec![0; 8].try_into().unwrap());
        round_trip::<VariableListU8<U8>>(vec![].try_into().unwrap());
    }

    #[test]
    fn ssz_empty_list() {
        let empty_list = VariableListU8::<U8>::default();
        let bytes = empty_list.as_ssz_bytes();
        assert!(bytes.is_empty());
        assert_eq!(VariableListU8::from_ssz_bytes(&[]).unwrap(), empty_list);
    }

    fn root_with_length(bytes: &[u8], len: usize) -> Hash256 {
        let root = merkle_root(bytes, 0);
        tree_hash::mix_in_length(&root, len)
    }

    #[test]
    fn tree_hash_u8() {
        let fixed: VariableListU8<U0> = VariableListU8::try_from(vec![]).unwrap();
        assert_eq!(fixed.tree_hash_root(), root_with_length(&[0; 8], 0));

        for i in 0..=1 {
            let fixed: VariableListU8<U1> = VariableListU8::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=8 {
            let fixed: VariableListU8<U8> = VariableListU8::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=13 {
            let fixed: VariableListU8<U13> = VariableListU8::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=16 {
            let fixed: VariableListU8<U16> = VariableListU8::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        let source: Vec<u8> = (0..16).collect();
        let fixed: VariableListU8<U16> = VariableListU8::try_from(source.clone()).unwrap();
        assert_eq!(fixed.tree_hash_root(), root_with_length(&source, 16));
    }


    #[test]
    fn large_list_pre_allocation() {
        use std::iter;
        use typenum::U1099511627776;

        struct WonkyIterator<I> {
            hint: usize,
            iter: I,
        }

        impl<I> Iterator for WonkyIterator<I>
        where
            I: Iterator,
        {
            type Item = I::Item;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, Some(self.hint))
            }
        }

        type N = U1099511627776;
        type List = VariableListU8<N>;

        let iter = iter::repeat(1).take(5);
        let wonky_iter = WonkyIterator {
            hint: N::to_usize() / 2,
            iter: iter.clone(),
        };

        assert_eq!(
            List::try_from_iter(iter).unwrap(),
            List::try_from_iter(wonky_iter).unwrap()
        );
    }

    #[test]
    fn std_hash() {
        let x: VariableListU8<U16> = VariableListU8::try_from(vec![3; 16]).unwrap();
        let y: VariableListU8<U16> = VariableListU8::try_from(vec![4; 16]).unwrap();
        let mut hashset = HashSet::new();

        for value in [x.clone(), y.clone()] {
            assert!(hashset.insert(value.clone()));
            assert!(!hashset.insert(value.clone()));
            assert!(hashset.contains(&value));
        }
        assert_eq!(hashset.len(), 2);
    }

    #[test]
    fn serde_invalid_length() {
        use typenum::U4;
        let json = serde_json::json!([1, 2, 3, 4, 5]);
        let result: Result<VariableListU8<U4>, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = serde_json::json!([1, 2, 3]);
        let result: Result<VariableListU8<U4>, _> = serde_json::from_value(json);
        assert!(result.is_ok());

        let json = serde_json::json!([1, 2, 3, 4]);
        let result: Result<VariableListU8<U4>, _> = serde_json::from_value(json);
        assert!(result.is_ok());
    }
}