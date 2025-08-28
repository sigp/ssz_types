use crate::tree_hash::vec_tree_hash_root;
use crate::{Error, FixedVector};
use serde::Deserialize;
use serde_derive::Serialize;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;
use tree_hash::Hash256;
use typenum::Unsigned;

pub use typenum;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct FixedVectorU8<N> {
    inner: FixedVector<u8, N>,
}

impl<N> PartialEq for FixedVectorU8<N> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<N> Eq for FixedVectorU8<N> {}
impl<N> std::hash::Hash for FixedVectorU8<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<N: Unsigned> FixedVectorU8<N> {
    pub fn new(vec: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            inner: FixedVector::new(vec)?,
        })
    }

    pub fn from_elem(elem: u8) -> Self {
        Self {
            inner: FixedVector::from_elem(elem),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn capacity() -> usize {
        FixedVector::<u8, N>::capacity()
    }
}

impl<N: Unsigned> TryFrom<Vec<u8>> for FixedVectorU8<N> {
    type Error = Error;

    fn try_from(vec: Vec<u8>) -> Result<Self, Error> {
        Self::new(vec)
    }
}

impl<N: Unsigned> From<FixedVectorU8<N>> for Vec<u8> {
    fn from(vector: FixedVectorU8<N>) -> Vec<u8> {
        vector.inner.into()
    }
}

impl<N: Unsigned> Default for FixedVectorU8<N> {
    fn default() -> Self {
        Self {
            inner: FixedVector::default(),
        }
    }
}

impl<N: Unsigned, I: SliceIndex<[u8]>> Index<I> for FixedVectorU8<N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.inner, index)
    }
}

impl<N: Unsigned, I: SliceIndex<[u8]>> IndexMut<I> for FixedVectorU8<N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.inner, index)
    }
}

impl<N: Unsigned> Deref for FixedVectorU8<N> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.inner
    }
}

impl<N: Unsigned> DerefMut for FixedVectorU8<N> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

impl<'a, N: Unsigned> IntoIterator for &'a FixedVectorU8<N> {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<N: Unsigned> IntoIterator for FixedVectorU8<N> {
    type Item = u8;
    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<N: Unsigned> tree_hash::TreeHash for FixedVectorU8<N> {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        tree_hash::TreeHashType::Vector
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        unreachable!("Vector should never be packed.")
    }

    fn tree_hash_packing_factor() -> usize {
        unreachable!("Vector should never be packed.")
    }

    fn tree_hash_root(&self) -> Hash256 {
        vec_tree_hash_root::<u8>(&self.inner, N::to_usize())
    }
}

impl<N: Unsigned> ssz::Encode for FixedVectorU8<N> {
    fn is_ssz_fixed_len() -> bool {
        u8::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        if <Self as ssz::Encode>::is_ssz_fixed_len() {
            u8::ssz_fixed_len() * N::to_usize()
        } else {
            ssz::BYTES_PER_LENGTH_OFFSET
        }
    }

    fn ssz_bytes_len(&self) -> usize {
        self.inner.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.inner.ssz_append(buf)
    }
}

impl<N: Unsigned> ssz::TryFromIter<u8> for FixedVectorU8<N> {
    type Error = Error;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = u8>,
    {
        let inner = FixedVector::try_from_iter(value)?;
        Ok(Self { inner })
    }
}

impl<N: Unsigned> ssz::Decode for FixedVectorU8<N> {
    fn is_ssz_fixed_len() -> bool {
        u8::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        if <Self as ssz::Decode>::is_ssz_fixed_len() {
            u8::ssz_fixed_len() * N::to_usize()
        } else {
            ssz::BYTES_PER_LENGTH_OFFSET
        }
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let inner = FixedVector::from_ssz_bytes(bytes)?;
        Ok(Self { inner })
    }
}

impl<'de, N> Deserialize<'de> for FixedVectorU8<N>
where
    N: Unsigned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = FixedVector::<u8, N>::deserialize(deserializer)?;
        Ok(Self { inner })
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, N: 'static + Unsigned> arbitrary::Arbitrary<'a> for FixedVectorU8<N> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let inner = FixedVector::<u8, N>::arbitrary(u)?;
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
        let fixed: Result<FixedVectorU8<U4>, _> = FixedVectorU8::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 3];
        let fixed: Result<FixedVectorU8<U4>, _> = FixedVectorU8::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 4];
        let fixed: Result<FixedVectorU8<U4>, _> = FixedVectorU8::new(vec);
        assert!(fixed.is_ok());
    }

    #[test]
    fn indexing() {
        let mut vec = vec![1, 2];
        vec.resize_with(8192, u8::default);

        let mut fixed: FixedVectorU8<U8192> = vec.clone().try_into().unwrap();

        assert_eq!(fixed[0], 1);
        assert_eq!(&fixed[0..1], &vec[0..1]);
        assert_eq!((fixed[..]).len(), 8192);

        fixed[1] = 3;
        assert_eq!(fixed[1], 3);
    }

    #[test]
    fn wrong_length() {
        let vec = vec![42; 5];
        let err = FixedVectorU8::<U4>::try_from(vec.clone()).unwrap_err();
        assert_eq!(err, Error::OutOfBounds { i: 5, len: 4 });

        let vec = vec![42; 3];
        let err = FixedVectorU8::<U4>::try_from(vec.clone()).unwrap_err();
        assert_eq!(err, Error::OutOfBounds { i: 3, len: 4 });

        let vec = vec![];
        let err = FixedVectorU8::<U4>::try_from(vec).unwrap_err();
        assert_eq!(err, Error::OutOfBounds { i: 0, len: 4 });
    }

    #[test]
    fn deref() {
        let vec = vec![0, 2, 4, 6];
        let fixed: FixedVectorU8<U4> = FixedVectorU8::try_from(vec).unwrap();

        assert_eq!(fixed.first(), Some(&0));
        assert_eq!(fixed.get(3), Some(&6));
        assert_eq!(fixed.get(4), None);
    }

    #[test]
    fn iterator() {
        let vec = vec![0, 2, 4, 6];
        let fixed: FixedVectorU8<U4> = FixedVectorU8::try_from(vec).unwrap();

        assert_eq!((&fixed).into_iter().sum::<u8>(), 12);
        assert_eq!(fixed.into_iter().sum::<u8>(), 12);
    }

    #[test]
    fn ssz_encode() {
        let vec: FixedVectorU8<U2> = vec![0; 2].try_into().unwrap();
        assert_eq!(vec.as_ssz_bytes(), vec![0, 0]);
        assert_eq!(<FixedVectorU8<U2> as Encode>::ssz_fixed_len(), 2);
    }

    fn ssz_round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn ssz_round_trip_u8_len_8() {
        ssz_round_trip::<FixedVectorU8<U8>>(vec![42; 8].try_into().unwrap());
        ssz_round_trip::<FixedVectorU8<U8>>(vec![0; 8].try_into().unwrap());
    }

    #[test]
    fn tree_hash_u8() {
        let fixed: FixedVectorU8<U0> = FixedVectorU8::try_from(vec![]).unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&[0; 8], 0));

        let fixed: FixedVectorU8<U1> = FixedVectorU8::try_from(vec![0; 1]).unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&[0; 8], 0));

        let fixed: FixedVectorU8<U8> = FixedVectorU8::try_from(vec![0; 8]).unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&[0; 8], 0));

        let fixed: FixedVectorU8<U16> = FixedVectorU8::try_from(vec![42; 16]).unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&[42; 16], 0));

        let source: Vec<u8> = (0..16).collect();
        let fixed: FixedVectorU8<U16> = FixedVectorU8::try_from(source.clone()).unwrap();
        assert_eq!(fixed.tree_hash_root(), merkle_root(&source, 0));
    }

    #[test]
    fn std_hash() {
        let x: FixedVectorU8<U16> = FixedVectorU8::try_from(vec![3; 16]).unwrap();
        let y: FixedVectorU8<U16> = FixedVectorU8::try_from(vec![4; 16]).unwrap();
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
        let result: Result<FixedVectorU8<U4>, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = serde_json::json!([1, 2, 3]);
        let result: Result<FixedVectorU8<U4>, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = serde_json::json!([1, 2, 3, 4]);
        let result: Result<FixedVectorU8<U4>, _> = serde_json::from_value(json);
        assert!(result.is_ok());
    }
}