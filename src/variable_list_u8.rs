use crate::{Error, VariableList};
use serde::Deserialize;
use serde_derive::Serialize;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;
use tree_hash::{merkle_root, Hash256};
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
        let root = merkle_root(self, 0);
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
    use std::collections::HashSet;
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
            vec![42; 5], // too long
            vec![42; 3], // within limits
            vec![42; 4], // at max length
            vec![],      // empty
            vec![1, 2, 3, 4], // at max length with different values
        ]);

        test_equivalent_behavior::<U0>(vec![
            vec![], // correct (empty)
            vec![1], // too long
        ]);

        // Test various lengths - comprehensive non-full testing
        test_equivalent_behavior::<U8>(vec![
            vec![],                           // empty
            vec![1],                          // length 1
            vec![1, 2],                       // length 2  
            vec![1, 2, 3],                    // length 3
            vec![1, 2, 3, 4],                 // length 4
            vec![1, 2, 3, 4, 5],              // length 5
            vec![1, 2, 3, 4, 5, 6],           // length 6
            vec![1, 2, 3, 4, 5, 6, 7],        // length 7
            vec![1, 2, 3, 4, 5, 6, 7, 8],     // at max length
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],  // too long
        ]);
        
        // Test non-full lists with different patterns
        test_equivalent_behavior::<U16>(vec![
            vec![],
            vec![255],                        // single byte, max value
            vec![0, 255],                     // min/max pair
            vec![128; 3],                     // repeated middle value
            vec![1, 2, 3, 4, 5],              // ascending sequence
            vec![5, 4, 3, 2, 1],              // descending sequence
            (0..10).collect(),                // length 10 (non-full)
            (0..15).collect(),                // length 15 (almost full)
            (0..16).collect(),                // length 16 (at max)
        ]);
    }

    #[test]
    fn indexing_and_deref() {
        let test_data = vec![0, 2, 4, 6];
        let original = VariableList::<u8, U4>::try_from(test_data.clone()).unwrap();
        let u8_variant = VariableListU8::<U4>::try_from(test_data).unwrap();

        // Test indexing
        assert_eq!(original[0], u8_variant[0]);
        assert_eq!(original[3], u8_variant[3]);
        assert_eq!(&original[0..2], &u8_variant[0..2]);

        // Test deref operations
        assert_eq!(original.first(), u8_variant.first());
        assert_eq!(original.last(), u8_variant.last());
        assert_eq!(original.get(3), u8_variant.get(3));
        assert_eq!(original.get(4), u8_variant.get(4));
    }

    #[test]
    fn mutable_operations() {
        let test_data = vec![1, 2, 3];
        let mut original = VariableList::<u8, U4>::try_from(test_data.clone()).unwrap();
        let mut u8_variant = VariableListU8::<U4>::try_from(test_data).unwrap();

        // Test mutable indexing
        original[1] = 99;
        u8_variant[1] = 99;
        assert_eq!(&original[..], &u8_variant[..]);

        // Test push operation
        let push_original = original.push(88);
        let push_u8 = u8_variant.push(88);
        
        match (push_original, push_u8) {
            (Ok(()), Ok(())) => {
                assert_eq!(&original[..], &u8_variant[..]);
                assert_eq!(original.len(), u8_variant.len());
            }
            (Err(original_err), Err(u8_err)) => {
                assert_eq!(original_err, u8_err);
            }
            _ => panic!("Push results should match"),
        }
    }

    #[test]
    fn push_behavior() {
        let mut original = VariableList::<u8, U3>::empty();
        let mut u8_variant = VariableListU8::<U3>::empty();

        // Push until full
        for i in 0..3 {
            let push_original = original.push(i);
            let push_u8 = u8_variant.push(i);
            assert_eq!(push_original, push_u8);
            assert!(push_original.is_ok());
            assert_eq!(&original[..], &u8_variant[..]);
        }

        // Try to push past capacity
        let push_original = original.push(99);
        let push_u8 = u8_variant.push(99);
        assert_eq!(push_original, push_u8);
        assert!(push_original.is_err());
    }

    #[test]
    fn iterator_behavior() {
        let test_data = vec![1, 2, 3, 4];
        let original = VariableList::<u8, U8>::try_from(test_data.clone()).unwrap();
        let u8_variant = VariableListU8::<U8>::try_from(test_data).unwrap();

        // Test borrowed iterator
        let original_sum: u8 = (&original).into_iter().sum();
        let u8_sum: u8 = (&u8_variant).into_iter().sum();
        assert_eq!(original_sum, u8_sum);

        // Test owned iterator (consume clones)
        let original_sum: u8 = original.clone().into_iter().sum();
        let u8_sum: u8 = u8_variant.clone().into_iter().sum();
        assert_eq!(original_sum, u8_sum);
    }

    #[test]
    fn ssz_encoding() {
        let test_vectors = vec![
            vec![], // empty
            vec![0],                          // length 1
            vec![0; 2],                       // length 2
            vec![255],                        // single max value
            vec![1, 2, 3],                    // length 3
            vec![42; 5],                      // length 5
            vec![42; 8],                      // length 8
            vec![255, 128, 64, 32],           // length 4 with varied values
            vec![0, 1, 2, 3, 4, 5, 6],        // length 7
            (0..10).collect(),                // length 10
            (0..12).collect(),                // length 12
            (0..16).collect(),                // length 16 (full)
        ];

        for test_data in test_vectors {
            let len = test_data.len();
            if len <= 16 {
                let original = VariableList::<u8, U16>::try_from(test_data.clone()).unwrap();
                let u8_variant = VariableListU8::<U16>::try_from(test_data).unwrap();
                
                assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                assert_eq!(original.ssz_bytes_len(), u8_variant.ssz_bytes_len());
                assert_eq!(<VariableList<u8, U16> as ssz::Encode>::is_ssz_fixed_len(), <VariableListU8<U16> as ssz::Encode>::is_ssz_fixed_len());
            }
        }
    }

    #[test]
    fn ssz_round_trip() {
        let test_vectors = vec![
            vec![],                    // empty
            vec![1],                   // length 1
            vec![1, 2, 3],             // length 3 (non-full)
            vec![1, 2, 3, 4],          // length 4 (non-full)
            vec![255, 0, 128],         // length 3 with edge values
            vec![42; 5],               // length 5 (non-full)
            vec![42; 8],               // length 8 (full)
        ];

        for test_data in test_vectors {
            let original = VariableList::<u8, U8>::try_from(test_data.clone()).unwrap();
            let u8_variant = VariableListU8::<U8>::try_from(test_data).unwrap();

            let original_encoded = original.as_ssz_bytes();
            let u8_encoded = u8_variant.as_ssz_bytes();
            assert_eq!(original_encoded, u8_encoded);

            let original_decoded = VariableList::<u8, U8>::from_ssz_bytes(&original_encoded).unwrap();
            let u8_decoded = VariableListU8::<U8>::from_ssz_bytes(&u8_encoded).unwrap();
            
            assert_eq!(&original_decoded[..], &u8_decoded[..]);
            assert_eq!(original, original_decoded);
            assert_eq!(u8_variant, u8_decoded);
        }
    }

    #[test]
    fn empty_list_handling() {
        let original_empty = VariableList::<u8, U8>::default();
        let u8_empty = VariableListU8::<U8>::default();
        
        assert_eq!(original_empty.len(), u8_empty.len());
        assert_eq!(original_empty.is_empty(), u8_empty.is_empty());
        assert_eq!(&original_empty[..], &u8_empty[..]);
        
        // Test SSZ encoding of empty lists
        let original_bytes = original_empty.as_ssz_bytes();
        let u8_bytes = u8_empty.as_ssz_bytes();
        assert_eq!(original_bytes, u8_bytes);
        assert!(original_bytes.is_empty());
        
        // Test decoding empty lists
        let original_decoded = VariableList::<u8, U8>::from_ssz_bytes(&[]).unwrap();
        let u8_decoded = VariableListU8::<U8>::from_ssz_bytes(&[]).unwrap();
        assert_eq!(original_decoded, original_empty);
        assert_eq!(u8_decoded, u8_empty);
    }

    #[test]
    fn tree_hash_consistency() {
        let test_vectors = vec![
            (vec![], 0),
            (vec![0], 1),
            (vec![0; 3], 8),              // non-full length 3
            (vec![42; 5], 8),             // non-full length 5
            (vec![0; 8], 8),              // full length 8
            (vec![1, 2, 3], 8),           // non-full with varied data
            (vec![255; 7], 8),            // non-full with max values
            (vec![42; 10], 16),           // non-full length 10
            (vec![42; 16], 16),           // full length 16
            ((0..5).collect(), 16),       // non-full ascending sequence
            ((0..12).collect(), 16),      // non-full longer sequence
            ((0..16).collect(), 16),      // full ascending sequence
        ];

        for (test_data, max_len) in test_vectors {
            match max_len {
                0 => {
                    let original = VariableList::<u8, U0>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U0>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                1 => {
                    let original = VariableList::<u8, U1>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U1>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                8 => {
                    let original = VariableList::<u8, U8>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U8>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                16 => {
                    let original = VariableList::<u8, U16>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U16>::try_from(test_data).unwrap();
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn hash_and_equality() {
        let test_data1 = vec![3; 8];
        let test_data2 = vec![4; 8];
        
        let original1 = VariableList::<u8, U16>::try_from(test_data1.clone()).unwrap();
        let original2 = VariableList::<u8, U16>::try_from(test_data2.clone()).unwrap();
        let u8_variant1 = VariableListU8::<U16>::try_from(test_data1).unwrap();
        let u8_variant2 = VariableListU8::<U16>::try_from(test_data2).unwrap();

        // Test equality
        assert_eq!(original1 == original2, u8_variant1 == u8_variant2);
        assert_ne!(original1, original2);
        assert_ne!(u8_variant1, u8_variant2);

        // Test equal cases
        let test_data3 = vec![3; 8];
        let original3 = VariableList::<u8, U16>::try_from(test_data3.clone()).unwrap();
        let u8_variant3 = VariableListU8::<U16>::try_from(test_data3).unwrap();
        assert_eq!(original1, original3);
        assert_eq!(u8_variant1, u8_variant3);

        // Test hash behavior
        let mut original_set = HashSet::new();
        let mut u8_set = HashSet::new();

        assert!(original_set.insert(original1.clone()));
        assert!(u8_set.insert(u8_variant1.clone()));
        assert!(!original_set.insert(original1.clone()));
        assert!(!u8_set.insert(u8_variant1.clone()));

        assert!(original_set.contains(&original1));
        assert!(u8_set.contains(&u8_variant1));

        assert_eq!(original_set.len(), u8_set.len());
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
            let original_result: Result<VariableList<u8, U4>, _> = serde_json::from_value(json_data.clone());
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
        let original_result: Result<VariableList<u8, U4>, _> = serde_json::from_value(json_too_long.clone());
        let u8_result: Result<VariableListU8<U4>, _> = serde_json::from_value(json_too_long);
        
        assert!(original_result.is_err());
        assert!(u8_result.is_err());
    }

    #[test]
    fn try_from_iter() {
        let test_vectors = vec![
            vec![],
            vec![1, 2, 3],
            vec![1, 2, 3, 4],
            vec![1, 2, 3, 4, 5], // Should fail for max_len 4
        ];

        for data in test_vectors {
            let original_result = VariableList::<u8, U4>::try_from_iter(data.clone());
            let u8_result = VariableListU8::<U4>::try_from_iter(data);
            
            match (original_result, u8_result) {
                (Ok(original), Ok(u8_variant)) => {
                    assert_eq!(&original[..], &u8_variant[..]);
                }
                (Err(original_err), Err(u8_err)) => {
                    assert_eq!(original_err, u8_err);
                }
                _ => panic!("Both should succeed or both should fail"),
            }
        }
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

        let iter = iter::repeat(1).take(5);
        let wonky_iter = WonkyIterator {
            hint: N::to_usize() / 2,
            iter: iter.clone(),
        };

        let original_result = VariableList::<u8, N>::try_from_iter(iter);
        let u8_result = VariableListU8::<N>::try_from_iter(wonky_iter);
        
        assert!(original_result.is_ok());
        assert!(u8_result.is_ok());
        assert_eq!(&original_result.unwrap()[..], &u8_result.unwrap()[..]);
    }

    #[test]
    fn max_len_property() {
        assert_eq!(VariableList::<u8, U4>::max_len(), VariableListU8::<U4>::max_len());
        assert_eq!(VariableList::<u8, U8>::max_len(), VariableListU8::<U8>::max_len());
        assert_eq!(VariableList::<u8, U16>::max_len(), VariableListU8::<U16>::max_len());
    }

    #[test]
    fn non_full_lists_comprehensive() {
        // Test a variety of non-full lists with different capacities and sizes
        let test_cases = vec![
            // (data, max_capacity)
            (vec![1], 4),
            (vec![1, 2], 4),
            (vec![1, 2, 3], 4),
            (vec![255], 8),
            (vec![0, 128, 255], 8),
            (vec![42; 5], 8),
            (vec![1, 2, 3, 4, 5, 6, 7], 8),
            (vec![100; 3], 16),
            ((0..7).collect(), 16),
            ((10..20).collect(), 16),
            (vec![0, 255, 0, 255, 0], 16),
        ];

        for (test_data, max_cap) in test_cases {
            match max_cap {
                4 => {
                    let original = VariableList::<u8, U4>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U4>::try_from(test_data.clone()).unwrap();
                    
                    // Verify the list is not at max capacity
                    assert!(original.len() < VariableList::<u8, U4>::max_len(), 
                           "Test data should be non-full: len={}, max={}", 
                           original.len(), VariableList::<u8, U4>::max_len());
                    
                    // Test all behaviors are equivalent
                    assert_eq!(&original[..], &u8_variant[..]);
                    assert_eq!(original.len(), u8_variant.len());
                    assert_eq!(original.is_empty(), u8_variant.is_empty());
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                }
                8 => {
                    let original = VariableList::<u8, U8>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U8>::try_from(test_data.clone()).unwrap();
                    
                    assert!(original.len() < VariableList::<u8, U8>::max_len());
                    assert_eq!(&original[..], &u8_variant[..]);
                    assert_eq!(original.len(), u8_variant.len());
                    assert_eq!(original.is_empty(), u8_variant.is_empty());
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                }
                16 => {
                    let original = VariableList::<u8, U16>::try_from(test_data.clone()).unwrap();
                    let u8_variant = VariableListU8::<U16>::try_from(test_data.clone()).unwrap();
                    
                    assert!(original.len() < VariableList::<u8, U16>::max_len());
                    assert_eq!(&original[..], &u8_variant[..]);
                    assert_eq!(original.len(), u8_variant.len());
                    assert_eq!(original.is_empty(), u8_variant.is_empty());
                    assert_eq!(original.tree_hash_root(), u8_variant.tree_hash_root());
                    assert_eq!(original.as_ssz_bytes(), u8_variant.as_ssz_bytes());
                }
                _ => {}
            }
        }
    }
}
