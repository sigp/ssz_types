use crate::tree_hash::vec_tree_hash_root;
use crate::typenum_helpers::to_usize;
use crate::Error;
use serde::Deserialize;
use serde_derive::Serialize;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;
use tree_hash::Hash256;
use typenum::Unsigned;

pub use typenum;

/// Emulates a SSZ `List`.
///
/// An ordered, heap-allocated, variable-length, homogeneous collection of `T`, with no more than
/// `N` values.
///
/// This struct is backed by a Rust `Vec` but constrained such that it must be instantiated with a
/// fixed number of elements and you may not add or remove elements, only modify.
///
/// The length of this struct is fixed at the type-level using
/// [typenum](https://crates.io/crates/typenum).
///
/// ## Example
///
/// ```
/// use ssz_types::{VariableList, typenum};
///
/// let base: Vec<u64> = vec![1, 2, 3, 4];
///
/// // Create a `VariableList` from a `Vec` that has the expected length.
/// let exact: VariableList<_, typenum::U4> = VariableList::from(base.clone());
/// assert_eq!(&exact[..], &[1, 2, 3, 4]);
///
/// // Create a `VariableList` from a `Vec` that is too long and the `Vec` is truncated.
/// let short: VariableList<_, typenum::U3> = VariableList::from(base.clone());
/// assert_eq!(&short[..], &[1, 2, 3]);
///
/// // Create a `VariableList` from a `Vec` that is shorter than the maximum.
/// let mut long: VariableList<_, typenum::U5> = VariableList::from(base);
/// assert_eq!(&long[..], &[1, 2, 3, 4]);
///
/// // Push a value to if it does not exceed the maximum
/// long.push(5).unwrap();
/// assert_eq!(&long[..], &[1, 2, 3, 4, 5]);
///
/// // Push a value to if it _does_ exceed the maximum.
/// assert!(long.push(6).is_err());
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct VariableList<T, N> {
    vec: Vec<T>,
    _phantom: PhantomData<N>,
}

// Implement comparison functions even if N doesn't implement PartialEq
impl<T: PartialEq, N> PartialEq for VariableList<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}
impl<T: Eq, N> Eq for VariableList<T, N> {}
impl<T: std::hash::Hash, N> std::hash::Hash for VariableList<T, N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.hash(state);
    }
}

/// Maximum number of elements to pre-allocate in `try_from_iter`.
///
/// Some variable lists have *very long* maximum lengths such that we can't actually fit them
/// in memory. This value is set to 128K with the expectation that any list with a large maximum
/// length (N) will contain at least a few thousand small values. i.e. we're targeting an
/// allocation around the 1MiB to 10MiB mark.
const MAX_ELEMENTS_TO_PRE_ALLOCATE: usize = 128 * (1 << 10);

impl<T, N: Unsigned> VariableList<T, N> {
    /// Returns `Some` if the given `vec` equals the fixed length of `Self`. Otherwise returns
    /// `None`.
    pub fn new(vec: Vec<T>) -> Result<Self, Error> {
        if vec.len() <= to_usize::<N>() {
            Ok(Self {
                vec,
                _phantom: PhantomData,
            })
        } else {
            Err(Error::OutOfBounds {
                i: vec.len(),
                len: Self::max_len(),
            })
        }
    }

    /// Create an empty list.
    pub fn empty() -> Self {
        Self {
            vec: vec![],
            _phantom: PhantomData,
        }
    }

    /// Returns the number of values presently in `self`.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// True if `self` does not contain any values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the type-level maximum length.
    pub fn max_len() -> usize {
        to_usize::<N>()
    }

    /// Appends `value` to the back of `self`.
    ///
    /// Returns `Err(())` when appending `value` would exceed the maximum length.
    pub fn push(&mut self, value: T) -> Result<(), Error> {
        if self.vec.len() < Self::max_len() {
            self.vec.push(value);
            Ok(())
        } else {
            Err(Error::OutOfBounds {
                i: self.vec.len() + 1,
                len: Self::max_len(),
            })
        }
    }
}

impl<T, N: Unsigned> From<Vec<T>> for VariableList<T, N> {
    fn from(mut vec: Vec<T>) -> Self {
        vec.truncate(to_usize::<N>());

        Self {
            vec,
            _phantom: PhantomData,
        }
    }
}

impl<T, N: Unsigned> From<VariableList<T, N>> for Vec<T> {
    fn from(list: VariableList<T, N>) -> Vec<T> {
        list.vec
    }
}

impl<T, N: Unsigned> Default for VariableList<T, N> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T, N: Unsigned, I: SliceIndex<[T]>> Index<I> for VariableList<T, N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, N: Unsigned, I: SliceIndex<[T]>> IndexMut<I> for VariableList<T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, N: Unsigned> Deref for VariableList<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<T, N: Unsigned> DerefMut for VariableList<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<'a, T, N: Unsigned> IntoIterator for &'a VariableList<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, N: Unsigned> IntoIterator for VariableList<T, N> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T, N: Unsigned> tree_hash::TreeHash for VariableList<T, N>
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
        let root = vec_tree_hash_root::<T, N>(&self.vec);

        tree_hash::mix_in_length(&root, self.len())
    }
}

impl<T, N: Unsigned> ssz::Encode for VariableList<T, N>
where
    T: ssz::Encode,
{
    fn is_ssz_fixed_len() -> bool {
        <Vec<T>>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <Vec<T>>::ssz_fixed_len()
    }

    fn ssz_bytes_len(&self) -> usize {
        self.vec.ssz_bytes_len()
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        self.vec.ssz_append(buf)
    }
}

impl<T, N: Unsigned> ssz::TryFromIter<T> for VariableList<T, N> {
    type Error = Error;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = T>,
    {
        let n = to_usize::<N>();
        let clamped_n = std::cmp::min(MAX_ELEMENTS_TO_PRE_ALLOCATE, n);
        let iter = value.into_iter();

        // Pre-allocate up to `N` elements based on the iterator size hint.
        let (_, opt_max_len) = iter.size_hint();
        let mut l = Self::new(Vec::with_capacity(
            opt_max_len.map_or(clamped_n, |max_len| std::cmp::min(clamped_n, max_len)),
        ))?;
        for item in iter {
            l.push(item)?;
        }
        Ok(l)
    }
}

impl<T, N> ssz::Decode for VariableList<T, N>
where
    T: ssz::Decode,
    N: Unsigned,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        let max_len = to_usize::<N>();

        if bytes.is_empty() {
            Ok(vec![].into())
        } else if T::is_ssz_fixed_len() {
            let num_items = bytes
                .len()
                .checked_div(T::ssz_fixed_len())
                .ok_or(ssz::DecodeError::ZeroLengthItem)?;

            if num_items > max_len {
                return Err(ssz::DecodeError::BytesInvalid(format!(
                    "VariableList of {} items exceeds maximum of {}",
                    num_items, max_len
                )));
            }

            bytes
                .chunks(T::ssz_fixed_len())
                .try_fold(Vec::with_capacity(num_items), |mut vec, chunk| {
                    vec.push(T::from_ssz_bytes(chunk)?);
                    Ok(vec)
                })
                .map(Into::into)
        } else {
            ssz::decode_list_of_variable_length_items(bytes, Some(max_len))
                .map(|vec: Vec<_>| vec.into())
        }
    }
}

impl<'de, T, N> Deserialize<'de> for VariableList<T, N>
where
    T: Deserialize<'de>,
    N: Unsigned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        if vec.len() <= to_usize::<N>() {
            Ok(VariableList {
                vec,
                _phantom: PhantomData,
            })
        } else {
            Err(serde::de::Error::custom(format!(
                "VariableList length {} exceeds maximum length {}",
                vec.len(),
                to_usize::<N>()
            )))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>, N: 'static + Unsigned> arbitrary::Arbitrary<'a>
    for VariableList<T, N>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let max_size = to_usize::<N>();
        let rand = usize::arbitrary(u)?;
        let size = std::cmp::min(rand, max_size);
        let mut vec: Vec<T> = Vec::with_capacity(size);
        for _ in 0..size {
            vec.push(<T>::arbitrary(u)?);
        }
        Ok(Self::new(vec).map_err(|_| arbitrary::Error::IncorrectFormat)?)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ssz::*;
    use std::collections::HashSet;
    use tree_hash::{merkle_root, TreeHash};
    use tree_hash_derive::TreeHash;
    use typenum::*;

    #[test]
    fn new() {
        let vec = vec![42; 5];
        let fixed: Result<VariableList<u64, U4>, _> = VariableList::new(vec);
        assert!(fixed.is_err());

        let vec = vec![42; 3];
        let fixed: Result<VariableList<u64, U4>, _> = VariableList::new(vec);
        assert!(fixed.is_ok());

        let vec = vec![42; 4];
        let fixed: Result<VariableList<u64, U4>, _> = VariableList::new(vec);
        assert!(fixed.is_ok());
    }

    #[test]
    fn indexing() {
        let vec = vec![1, 2];

        let mut fixed: VariableList<u64, U8192> = vec.clone().into();

        assert_eq!(fixed[0], 1);
        assert_eq!(&fixed[0..1], &vec[0..1]);
        assert_eq!((fixed[..]).len(), 2);

        fixed[1] = 3;
        assert_eq!(fixed[1], 3);
    }

    #[test]
    fn length() {
        let vec = vec![42; 5];
        let fixed: VariableList<u64, U4> = VariableList::from(vec.clone());
        assert_eq!(&fixed[..], &vec[0..4]);

        let vec = vec![42; 3];
        let fixed: VariableList<u64, U4> = VariableList::from(vec.clone());
        assert_eq!(&fixed[0..3], &vec[..]);
        assert_eq!(&fixed[..], &vec![42, 42, 42][..]);

        let vec = vec![];
        let fixed: VariableList<u64, U4> = VariableList::from(vec);
        assert_eq!(&fixed[..], &[] as &[u64]);
    }

    #[test]
    fn deref() {
        let vec = vec![0, 2, 4, 6];
        let fixed: VariableList<u64, U4> = VariableList::from(vec);

        assert_eq!(fixed.first(), Some(&0));
        assert_eq!(fixed.get(3), Some(&6));
        assert_eq!(fixed.get(4), None);
    }

    #[test]
    fn encode() {
        let vec: VariableList<u16, U2> = vec![0; 2].into();
        assert_eq!(vec.as_ssz_bytes(), vec![0, 0, 0, 0]);
        assert_eq!(<VariableList<u16, U2> as Encode>::ssz_fixed_len(), 4);
    }

    fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn u16_len_8() {
        round_trip::<VariableList<u16, U8>>(vec![42; 8].into());
        round_trip::<VariableList<u16, U8>>(vec![0; 8].into());
    }

    fn root_with_length(bytes: &[u8], len: usize) -> Hash256 {
        let root = merkle_root(bytes, 0);
        tree_hash::mix_in_length(&root, len)
    }

    #[test]
    fn tree_hash_u8() {
        let fixed: VariableList<u8, U0> = VariableList::from(vec![]);
        assert_eq!(fixed.tree_hash_root(), root_with_length(&[0; 8], 0));

        for i in 0..=1 {
            let fixed: VariableList<u8, U1> = VariableList::from(vec![0; i]);
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=8 {
            let fixed: VariableList<u8, U8> = VariableList::from(vec![0; i]);
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=13 {
            let fixed: VariableList<u8, U13> = VariableList::from(vec![0; i]);
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=16 {
            let fixed: VariableList<u8, U16> = VariableList::from(vec![0; i]);
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        let source: Vec<u8> = (0..16).collect();
        let fixed: VariableList<u8, U16> = VariableList::from(source.clone());
        assert_eq!(fixed.tree_hash_root(), root_with_length(&source, 16));
    }

    #[derive(Clone, Copy, TreeHash, Default)]
    struct A {
        a: u32,
        b: u32,
    }

    fn repeat(input: &[u8], n: usize) -> Vec<u8> {
        let mut output = vec![];

        for _ in 0..n {
            output.append(&mut input.to_vec());
        }

        output
    }

    fn padded_root_with_length(bytes: &[u8], len: usize, min_nodes: usize) -> Hash256 {
        let root = merkle_root(bytes, min_nodes);
        tree_hash::mix_in_length(&root, len)
    }

    #[test]
    fn tree_hash_composite() {
        let a = A { a: 0, b: 1 };

        let fixed: VariableList<A, U0> = VariableList::from(vec![]);
        assert_eq!(
            fixed.tree_hash_root(),
            padded_root_with_length(&[0; 32], 0, 0),
        );

        for i in 0..=1 {
            let fixed: VariableList<A, U1> = VariableList::from(vec![a; i]);
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 1),
                "U1 {}",
                i
            );
        }

        for i in 0..=8 {
            let fixed: VariableList<A, U8> = VariableList::from(vec![a; i]);
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 8),
                "U8 {}",
                i
            );
        }

        for i in 0..=13 {
            let fixed: VariableList<A, U13> = VariableList::from(vec![a; i]);
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 13),
                "U13 {}",
                i
            );
        }

        for i in 0..=16 {
            let fixed: VariableList<A, U16> = VariableList::from(vec![a; i]);
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 16),
                "U16 {}",
                i
            );
        }
    }

    fn large_list_pre_allocation() {
        use std::iter;
        use typenum::U1099511627776;

        // Iterator that hints the upper bound on its length as `hint`.
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

        // Very large list type that would not fit in memory.
        type N = U1099511627776;
        type List = VariableList<u64, N>;

        let iter = iter::repeat(1).take(5);
        let wonky_iter = WonkyIterator {
            hint: to_usize::<N>() / 2,
            iter: iter.clone(),
        };

        // Don't explode.
        assert_eq!(
            List::try_from_iter(iter).unwrap(),
            List::try_from_iter(wonky_iter).unwrap()
        );
    }

    #[test]
    #[cfg(any(target_pointer_width = "64", feature = "cap-typenum-to-usize-overflow"))]
    fn large_list_pre_allocation_test() {
        large_list_pre_allocation()
    }

    #[test]
    #[cfg(not(any(target_pointer_width = "64", feature = "cap-typenum-to-usize-overflow")))]
    #[should_panic]
    fn large_list_pre_allocation_test() {
        large_list_pre_allocation()
    }

    #[test]
    fn std_hash() {
        let x: VariableList<u32, U16> = VariableList::from(vec![3; 16]);
        let y: VariableList<u32, U16> = VariableList::from(vec![4; 16]);
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
        let result: Result<VariableList<u64, U4>, _> = serde_json::from_value(json);
        assert!(result.is_err());

        let json = serde_json::json!([1, 2, 3]);
        let result: Result<VariableList<u64, U4>, _> = serde_json::from_value(json);
        assert!(result.is_ok());

        let json = serde_json::json!([1, 2, 3, 4]);
        let result: Result<VariableList<u64, U4>, _> = serde_json::from_value(json);
        assert!(result.is_ok());
    }

    mod large_typenums {
        use crate::tree_hash::packing_factor;

        use super::*;
        use ethereum_hashing::ZERO_HASHES;
        fn sanity_check() {
            assert_eq!(U1099511627776::to_u64(), 1099511627776u64);
        }

        fn ssz_bytes_len() {
            let vec: VariableList<u16, U8> = vec![0; 4].into();
            let vec2: VariableList<u16, U1099511627776> = vec![0; 4].into();
            assert_eq!(vec.len(), 4);
            assert_eq!(vec.ssz_bytes_len(), 8);
            assert_eq!(vec2.len(), 4);
            assert_eq!(vec2.ssz_bytes_len(), 8);
        }

        fn encode() {
            let vec: VariableList<u16, U1099511627776> = vec![0; 2].into();
            assert_eq!(vec.as_ssz_bytes(), vec![0, 0, 0, 0]);
            assert_eq!(
                <VariableList<u16, U1099511627776> as Encode>::ssz_fixed_len(),
                4
            );
        }

        fn encode_non_power_of_2() {
            type NVal = typenum::Add1<U1099511627776>;
            let vec: VariableList<u16, NVal> = vec![0; 2].into();
            assert_eq!(vec.as_ssz_bytes(), vec![0, 0, 0, 0]);
            assert_eq!(<VariableList<u16, NVal> as Encode>::ssz_fixed_len(), 4);
        }

        fn u16_len_2power40() {
            round_trip::<VariableList<u16, U1099511627776>>(vec![42; 8].into());
            round_trip::<VariableList<u16, U1099511627776>>(vec![0; 8].into());
        }

        trait CreateZero {
            fn zero() -> Self;
        }

        impl CreateZero for Hash256 {
            fn zero() -> Self {
                [0; 32].into()
            }
        }

        impl CreateZero for u64 {
            fn zero() -> Self {
                0
            }
        }

        struct TreeHashTestCase<T: TreeHash, N: Unsigned> {
            pub expected_hash: Hash256,
            pub vec: VariableList<T, N>,
        }

        impl<T: TreeHash + CreateZero + Clone, N: Unsigned> TreeHashTestCase<T, N> {
            fn test(&self) {
                assert_eq!(
                    self.vec.tree_hash_root(),
                    tree_hash::mix_in_length(&self.expected_hash, self.vec.len())
                );
            }
            pub fn zeros(vec_len: usize) -> Self {
                let full_depth = N::to_u64().next_power_of_two().ilog2();
                let packing_depth_discount = packing_factor::<T>().next_power_of_two().ilog2();
                let depth = (full_depth - packing_depth_discount) as usize;
                Self {
                    vec: VariableList::from(vec![T::zero(); vec_len]),
                    expected_hash: ZERO_HASHES[depth].into(),
                }
            }
        }

        struct AllTreeHashTests<T> {
            _phantom: PhantomData<T>,
        }

        impl<T: TreeHash + CreateZero + Clone> AllTreeHashTests<T> {
            pub fn all_tests() {
                TreeHashTestCase::<T, U16>::zeros(10).test();

                TreeHashTestCase::<T, Add1<U16>>::zeros(10).test();
                TreeHashTestCase::<T, Sub1<U32>>::zeros(10).test();
                TreeHashTestCase::<T, U32>::zeros(10).test();

                TreeHashTestCase::<T, U1024>::zeros(10).test();

                TreeHashTestCase::<T, Sub1<U65536>>::zeros(10).test();
                TreeHashTestCase::<T, U65536>::zeros(10).test();
                TreeHashTestCase::<T, Add1<U65536>>::zeros(10).test();

                TreeHashTestCase::<T, Sub1<U536870912>>::zeros(10).test();
                TreeHashTestCase::<T, U536870912>::zeros(10).test();
                TreeHashTestCase::<T, Add1<U536870912>>::zeros(10).test();

                TreeHashTestCase::<T, Sub1<U1073741824>>::zeros(10).test();
                TreeHashTestCase::<T, U1073741824>::zeros(10).test();
                TreeHashTestCase::<T, Add1<U1073741824>>::zeros(10).test();

                TreeHashTestCase::<T, Sub1<U2147483648>>::zeros(10).test();
                TreeHashTestCase::<T, U2147483648>::zeros(10).test();
                TreeHashTestCase::<T, Add1<U2147483648>>::zeros(10).test();

                TreeHashTestCase::<T, Sub1<U4294967296>>::zeros(10).test();
                TreeHashTestCase::<T, U4294967296>::zeros(10).test();
                TreeHashTestCase::<T, Add1<U4294967296>>::zeros(10).test();

                // 2 ** 40
                TreeHashTestCase::<T, Sub1<U1099511627776>>::zeros(10).test();
                TreeHashTestCase::<T, U1099511627776>::zeros(10).test();
                TreeHashTestCase::<T, Add1<U1099511627776>>::zeros(10).test();

                // 2**48
                TreeHashTestCase::<T, Sub1<U281474976710656>>::zeros(10).test();
                TreeHashTestCase::<T, U281474976710656>::zeros(10).test();
                // Beyond 2**48 target_pointer_width="64" arches still work ok, target_pointer_width="32" fail due to ethereum_hashing::ZEROHASHES running out of elements
            }
        }

        #[cfg(any(target_pointer_width = "64", feature = "cap-typenum-to-usize-overflow"))]
        mod arch64_bit_or_capping {
            #[test]
            fn sanity_check() {
                super::sanity_check()
            }

            #[test]
            fn ssz_bytes_len() {
                super::ssz_bytes_len();
            }

            #[test]
            fn encode() {
                super::encode();
            }

            #[test]
            fn encode_non_power_of_2() {
                super::encode_non_power_of_2();
            }

            #[test]
            fn u16_len_2power40() {
                super::u16_len_2power40()
            }

            #[test]
            fn tree_hash_tests_hash256() {
                super::AllTreeHashTests::<super::Hash256>::all_tests();
            }

            #[test]
            fn tree_hash_tests_u64() {
                super::AllTreeHashTests::<u64>::all_tests();
            }
        }

        #[cfg(not(any(target_pointer_width = "64", feature = "cap-typenum-to-usize-overflow")))]
        mod arch32_bit_no_capping {
            #[test]
            fn sanity_check() {
                super::sanity_check()
            }

            #[test]
            #[should_panic()]
            fn ssz_bytes_len() {
                super::ssz_bytes_len();
            }

            #[test]
            #[should_panic()]
            fn encode() {
                super::encode();
            }

            #[test]
            #[should_panic()]
            fn encode_non_power_of_2() {
                super::encode_non_power_of_2();
            }

            #[test]
            #[should_panic()]
            fn u16_len_2power40() {
                super::u16_len_2power40()
            }

            #[test]
            #[should_panic()]
            fn tree_hash_tests_hash256() {
                super::AllTreeHashTests::<super::Hash256>::all_tests();
            }

            #[test]
            #[should_panic()]
            fn tree_hash_tests_u64() {
                super::AllTreeHashTests::<u64>::all_tests();
            }
        }
    }
}
