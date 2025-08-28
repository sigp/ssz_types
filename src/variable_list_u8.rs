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
        false
    }

    fn ssz_fixed_len() -> usize {
        ssz::BYTES_PER_LENGTH_OFFSET
    }

    fn ssz_bytes_len(&self) -> usize {
        self.len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.inner);
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
        VariableList::new(bytes.to_vec())
            .map(|inner| Self { inner })
            .map_err(|e| ssz::DecodeError::BytesInvalid(format!("{e:?}")))
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
    use tree_hash::TreeHash;
    use typenum::*;

    fn test_equivalent_behavior<N: Unsigned>(test_vectors: Vec<Vec<u8>>) {
        for vec in test_vectors {
            let original_result = VariableList::<u8, N>::new(vec.clone());
            let u8_result = VariableListU8::<N>::new(vec);

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
            vec![42; 3],      // within limits
            vec![42; 4],      // at max length
            vec![],           // empty
            vec![1, 2, 3, 4], // at max length with different values
        ]);

        test_equivalent_behavior::<U0>(vec![
            vec![],  // correct (empty)
            vec![1], // too long
        ]);

        // Test various lengths - comprehensive non-full testing
        test_equivalent_behavior::<U8>(vec![
            vec![],                          // empty
            vec![1],                         // length 1
            vec![1, 2],                      // length 2
            vec![1, 2, 3],                   // length 3
            vec![1, 2, 3, 4],                // length 4
            vec![1, 2, 3, 4, 5],             // length 5
            vec![1, 2, 3, 4, 5, 6],          // length 6
            vec![1, 2, 3, 4, 5, 6, 7],       // length 7
            vec![1, 2, 3, 4, 5, 6, 7, 8],    // at max length
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9], // too long
        ]);

        // Test non-full lists with different patterns
        test_equivalent_behavior::<U16>(vec![
            vec![],
            vec![255],           // single byte, max value
            vec![0, 255],        // min/max pair
            vec![128; 3],        // repeated middle value
            vec![1, 2, 3, 4, 5], // ascending sequence
            vec![5, 4, 3, 2, 1], // descending sequence
            (0..10).collect(),   // length 10 (non-full)
            (0..15).collect(),   // length 15 (almost full)
            (0..16).collect(),   // length 16 (at max)
        ]);
    }

    #[test]
    fn ssz_encoding() {
        let test_vectors = vec![
            vec![],                    // empty
            vec![0],                   // length 1
            vec![0; 2],                // length 2
            vec![255],                 // single max value
            vec![1, 2, 3],             // length 3
            vec![42; 5],               // length 5
            vec![42; 8],               // length 8
            vec![255, 128, 64, 32],    // length 4 with varied values
            vec![0, 1, 2, 3, 4, 5, 6], // length 7
            (0..10).collect(),         // length 10
            (0..12).collect(),         // length 12
            (0..16).collect(),         // length 16 (full)
        ];

        for test_data in test_vectors {
            let len = test_data.len();
            if len <= 16 {
                let original = VariableList::<u8, U16>::try_from(test_data.clone()).unwrap();
                let u8_variant = VariableListU8::<U16>::try_from(test_data).unwrap();

                assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                assert_eq!(original.ssz_bytes_len(), u8_variant.ssz_bytes_len());
                assert_eq!(
                    <VariableList<u8, U16> as ssz::Encode>::is_ssz_fixed_len(),
                    <VariableListU8<U16> as ssz::Encode>::is_ssz_fixed_len()
                );
            }
        }
    }

    #[test]
    fn ssz_round_trip() {
        let test_vectors = vec![
            vec![],            // empty
            vec![1],           // length 1
            vec![1, 2, 3],     // length 3 (non-full)
            vec![1, 2, 3, 4],  // length 4 (non-full)
            vec![255, 0, 128], // length 3 with edge values
            vec![42; 5],       // length 5 (non-full)
            vec![42; 8],       // length 8 (full)
        ];

        for test_data in test_vectors {
            let original = VariableList::<u8, U8>::try_from(test_data.clone()).unwrap();
            let u8_variant = VariableListU8::<U8>::try_from(test_data).unwrap();

            let original_encoded = original.as_ssz_bytes();
            let u8_encoded = u8_variant.as_ssz_bytes();
            assert_eq!(original_encoded, u8_encoded);

            let original_decoded =
                VariableList::<u8, U8>::from_ssz_bytes(&original_encoded).unwrap();
            let u8_decoded = VariableListU8::<U8>::from_ssz_bytes(&u8_encoded).unwrap();

            assert_eq!(&original_decoded[..], &u8_decoded[..]);
            assert_eq!(original, original_decoded);
            assert_eq!(u8_variant, u8_decoded);
        }
    }

    #[test]
    fn tree_hash_consistency() {
        // Tree hashing uses 32-byte leaves, so test around these boundaries
        // Format: (test_data, max_capacity)
        let test_vectors = vec![
            // Small sizes
            (vec![], 1), // 0 bytes, max 1
            (vec![0], 2), // 1 byte, max 2
            (vec![0; 8], 16), // 8 bytes, max 16
            (vec![0; 16], 32), // 16 bytes, max 32
            
            // Around 32-byte boundary (1 leaf)
            (vec![42; 30], 64), // 30 bytes (under 1 leaf)
            (vec![42; 31], 64), // 31 bytes (just under 1 leaf)
            (vec![42; 32], 64), // 32 bytes (exactly 1 leaf)
            (vec![255; 33], 64), // 33 bytes (just over 1 leaf)
            (vec![128; 35], 64), // 35 bytes (over 1 leaf)
            
            // Around 64-byte boundary (2 leaves)
            (vec![1; 62], 96), // 62 bytes (under 2 leaves)
            (vec![2; 63], 96), // 63 bytes (just under 2 leaves)
            (vec![3; 64], 96), // 64 bytes (exactly 2 leaves)
            (vec![4; 65], 96), // 65 bytes (just over 2 leaves)
            (vec![5; 67], 96), // 67 bytes (over 2 leaves)
            
            // Around 96-byte boundary (3 leaves)
            ((0..94).map(|i| (i % 256) as u8).collect(), 128), // 94 bytes (under 3 leaves)
            ((0..95).map(|i| (i % 256) as u8).collect(), 128), // 95 bytes (just under 3 leaves)
            ((0..96).map(|i| (i % 256) as u8).collect(), 128), // 96 bytes (exactly 3 leaves)
            ((0..97).map(|i| (i % 256) as u8).collect(), 128), // 97 bytes (just over 3 leaves)
            ((0..99).map(|i| (i % 256) as u8).collect(), 128), // 99 bytes (over 3 leaves)
            
            // Around 128-byte boundary (4 leaves)
            (vec![200; 126], 160), // 126 bytes (under 4 leaves)
            (vec![201; 127], 160), // 127 bytes (just under 4 leaves)
            (vec![202; 128], 160), // 128 bytes (exactly 4 leaves)
            (vec![203; 129], 160), // 129 bytes (just over 4 leaves)
            ((100..230).map(|i| (i % 256) as u8).collect(), 160), // 130 bytes (over 4 leaves)
            
            // Larger boundaries - 160 bytes (5 leaves)
            ((0..158).map(|i| (i % 256) as u8).collect(), 192), // 158 bytes (under 5 leaves)
            ((0..160).map(|i| (i % 256) as u8).collect(), 192), // 160 bytes (exactly 5 leaves)
            ((0..162).map(|i| (i % 256) as u8).collect(), 192), // 162 bytes (over 5 leaves)
            
            // Test non-full at various capacities
            (vec![42; 3], 64),   // small non-full
            (vec![99; 50], 128), // mid-size non-full
            (vec![77; 100], 192), // larger non-full
        ];

        for (test_data, max_cap) in test_vectors {
            match max_cap {
                1 => {
                    let original = VariableList::<u8, U1>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U1>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                2 => {
                    let original = VariableList::<u8, U2>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U2>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                16 => {
                    let original = VariableList::<u8, U16>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U16>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                32 => {
                    let original = VariableList::<u8, U32>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U32>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                64 => {
                    let original = VariableList::<u8, U64>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U64>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                96 => {
                    let original = VariableList::<u8, U96>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U96>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                128 => {
                    let original = VariableList::<u8, U128>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U128>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                160 => {
                    let original = VariableList::<u8, U160>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U160>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                192 => {
                    let original = VariableList::<u8, U192>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U192>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn serde_behavior() {
        // Test successful deserialization with various lengths
        for json_data in [
            serde_json::json!([]),
            serde_json::json!([1]),
            serde_json::json!([1, 2, 3]),
            serde_json::json!([1, 2, 3, 4]),
        ] {
            let original_result: Result<VariableList<u8, U4>, _> =
                serde_json::from_value(json_data.clone());
            let u8_result: Result<VariableListU8<U4>, _> = serde_json::from_value(json_data);

            match (original_result, u8_result) {
                (Ok(original), Ok(u8_variant)) => {
                    assert_eq!(&original[..], &u8_variant[..]);
                }
                (Err(_), Err(_)) => {} // Both should fail in same cases
                _ => panic!("Serde results should match"),
            }
        }

        // Test failed deserialization - too long
        let json_too_long = serde_json::json!([1, 2, 3, 4, 5]);
        let original_result: Result<VariableList<u8, U4>, _> =
            serde_json::from_value(json_too_long.clone());
        let u8_result: Result<VariableListU8<U4>, _> = serde_json::from_value(json_too_long);

        assert!(original_result.is_err());
        assert!(u8_result.is_err());
    }
}
