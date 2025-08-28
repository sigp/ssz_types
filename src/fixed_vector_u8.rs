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
        true
    }

    fn ssz_fixed_len() -> usize {
        N::to_usize()
    }

    fn ssz_bytes_len(&self) -> usize {
        self.len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.inner);
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
        true
    }

    fn ssz_fixed_len() -> usize {
        N::to_usize()
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        FixedVector::new(bytes.to_vec())
            .map(|inner| Self { inner })
            .map_err(|e| ssz::DecodeError::BytesInvalid(format!("{e:?}")))
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
    use tree_hash::TreeHash;
    use typenum::*;

    fn test_equivalent_behavior<N: Unsigned>(test_vectors: Vec<Vec<u8>>) {
        for vec in test_vectors {
            let original_result = FixedVector::<u8, N>::new(vec.clone());
            let u8_result = FixedVectorU8::<N>::new(vec);

            match (original_result, u8_result) {
                (Ok(original), Ok(u8_variant)) => {
                    // Test basic properties
                    assert_eq!(original.len(), u8_variant.len());
                    assert_eq!(original.is_empty(), u8_variant.is_empty());
                    assert_eq!(&original[..], &u8_variant[..]);

                    // Test trait implementations
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                    assert_eq!(original.ssz_bytes_len(), u8_variant.ssz_bytes_len());

                    // Test conversion back to Vec
                    let original_vec: Vec<u8> = original.into();
                    let u8_vec: Vec<u8> = u8_variant.into();
                    assert_eq!(original_vec, u8_vec);
                }
                (Err(original_err), Err(u8_err)) => {
                    assert_eq!(original_err, u8_err);
                }
                _ => panic!("Results should both succeed or both fail"),
            }
        }
    }

    #[test]
    fn construction_and_basic_operations() {
        test_equivalent_behavior::<U4>(vec![
            vec![42; 5],      // too long
            vec![42; 3],      // too short
            vec![42; 4],      // correct length
            vec![],           // empty (too short)
            vec![1, 2, 3, 4], // correct length with different values
        ]);

        test_equivalent_behavior::<U0>(vec![
            vec![],  // correct (empty)
            vec![1], // too long
        ]);
    }

    #[test]
    fn ssz_encoding() {
        let test_vectors = vec![
            vec![0; 2],
            vec![42; 8],
            vec![255, 128, 64, 32],
            (0..16).collect(),
        ];

        for test_data in test_vectors {
            let len = test_data.len();
            match len {
                2 => {
                    let original = FixedVector::<u8, U2>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U2>::try_from(test_data).unwrap();

                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                    assert_eq!(original.ssz_bytes_len(), u8_variant.ssz_bytes_len());
                    assert_eq!(
                        <FixedVector<u8, U2> as ssz::Encode>::ssz_fixed_len(),
                        <FixedVectorU8<U2> as ssz::Encode>::ssz_fixed_len()
                    );
                }
                4 => {
                    let original = FixedVector::<u8, U4>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U4>::try_from(test_data).unwrap();

                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                    assert_eq!(original.ssz_bytes_len(), u8_variant.ssz_bytes_len());
                }
                8 => {
                    let original = FixedVector::<u8, U8>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U8>::try_from(test_data).unwrap();

                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                    assert_eq!(original.ssz_bytes_len(), u8_variant.ssz_bytes_len());
                }
                16 => {
                    let original = FixedVector::<u8, U16>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U16>::try_from(test_data).unwrap();

                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                    assert_eq!(original.ssz_bytes_len(), u8_variant.ssz_bytes_len());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn ssz_round_trip() {
        let test_data = vec![42; 8];
        let original = FixedVector::<u8, U8>::try_from(test_data.clone()).unwrap();
        let u8_variant = FixedVectorU8::<U8>::try_from(test_data).unwrap();

        let original_encoded = original.as_ssz_bytes();
        let u8_encoded = u8_variant.as_ssz_bytes();
        assert_eq!(original_encoded, u8_encoded);

        let original_decoded = FixedVector::<u8, U8>::from_ssz_bytes(&original_encoded).unwrap();
        let u8_decoded = FixedVectorU8::<U8>::from_ssz_bytes(&u8_encoded).unwrap();

        assert_eq!(&original_decoded[..], &u8_decoded[..]);
        assert_eq!(original, original_decoded);
        assert_eq!(u8_variant, u8_decoded);
    }

    #[test]
    fn tree_hash_consistency() {
        // Tree hashing uses 32-byte leaves, so test around these boundaries
        let test_vectors = vec![
            vec![], // 0 bytes
            vec![0], // 1 byte
            vec![0; 8], // 8 bytes
            vec![0; 16], // 16 bytes
            vec![0; 31], // 31 bytes (just under 32)
            vec![0; 32], // 32 bytes (exactly one leaf)
            vec![0; 33], // 33 bytes (just over one leaf)
            vec![42; 63], // 63 bytes (just under 2 leaves)
            vec![42; 64], // 64 bytes (exactly 2 leaves)
            vec![42; 65], // 65 bytes (just over 2 leaves)
            vec![255; 95], // 95 bytes (just under 3 leaves)
            vec![255; 96], // 96 bytes (exactly 3 leaves)
            vec![128; 128], // 128 bytes (exactly 4 leaves)
            (0..160).map(|i| (i % 256) as u8).collect(), // 160 bytes (5 leaves)
        ];

        for test_data in test_vectors {
            let len = test_data.len();
            match len {
                0 => {
                    let original = FixedVector::<u8, U0>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U0>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                1 => {
                    let original = FixedVector::<u8, U1>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U1>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                8 => {
                    let original = FixedVector::<u8, U8>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U8>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                16 => {
                    let original = FixedVector::<u8, U16>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U16>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                31 => {
                    let original = FixedVector::<u8, U31>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U31>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                32 => {
                    let original = FixedVector::<u8, U32>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U32>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                33 => {
                    let original = FixedVector::<u8, U33>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U33>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                63 => {
                    let original = FixedVector::<u8, U63>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U63>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                64 => {
                    let original = FixedVector::<u8, U64>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U64>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                65 => {
                    let original = FixedVector::<u8, U65>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U65>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                95 => {
                    let original = FixedVector::<u8, U95>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U95>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                96 => {
                    let original = FixedVector::<u8, U96>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U96>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                128 => {
                    let original = FixedVector::<u8, U128>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U128>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                160 => {
                    let original = FixedVector::<u8, U160>::try_from(test_data.clone()).unwrap();
                    let u8_variant = FixedVectorU8::<U160>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn serde_behavior() {
        // Test successful deserialization
        let json_valid = serde_json::json!([1, 2, 3, 4]);
        let original_result: Result<FixedVector<u8, U4>, _> =
            serde_json::from_value(json_valid.clone());
        let u8_result: Result<FixedVectorU8<U4>, _> = serde_json::from_value(json_valid);

        assert!(original_result.is_ok());
        assert!(u8_result.is_ok());
        assert_eq!(&original_result.unwrap()[..], &u8_result.unwrap()[..]);

        // Test failed deserialization - too long
        let json_too_long = serde_json::json!([1, 2, 3, 4, 5]);
        let original_result: Result<FixedVector<u8, U4>, _> =
            serde_json::from_value(json_too_long.clone());
        let u8_result: Result<FixedVectorU8<U4>, _> = serde_json::from_value(json_too_long);

        assert!(original_result.is_err());
        assert!(u8_result.is_err());

        // Test failed deserialization - too short
        let json_too_short = serde_json::json!([1, 2, 3]);
        let original_result: Result<FixedVector<u8, U4>, _> =
            serde_json::from_value(json_too_short.clone());
        let u8_result: Result<FixedVectorU8<U4>, _> = serde_json::from_value(json_too_short);

        assert!(original_result.is_err());
        assert!(u8_result.is_err());
    }
}
