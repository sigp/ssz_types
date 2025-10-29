use crate::tree_hash::vec_tree_hash_root;
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
/// use ssz_types::{Error, VariableList, typenum};
///
/// let base: Vec<u64> = vec![1, 2, 3, 4];
///
/// // Create a `VariableList` from a `Vec` that has the expected length.
/// let exact: VariableList<_, typenum::U4> = VariableList::try_from(base.clone()).unwrap();
/// assert_eq!(&exact[..], &[1, 2, 3, 4]);
///
/// // Create a `VariableList` from a `Vec` that is too long and you will get an error.
/// let err = VariableList::<_, typenum::U3>::try_from(base.clone()).unwrap_err();
/// assert_eq!(err, Error::OutOfBounds { i: 4, len: 3 });
///
/// // Create a `VariableList` from a `Vec` that is shorter than the maximum.
/// let mut long: VariableList<_, typenum::U5> = VariableList::try_from(base).unwrap();
/// assert_eq!(&long[..], &[1, 2, 3, 4]);
///
/// // Push a value to if it does not exceed the maximum
/// long.push(5).unwrap();
/// assert_eq!(&long[..], &[1, 2, 3, 4, 5]);
///
/// // Push a value to if it _does_ exceed the maximum.
/// assert!(long.push(6).is_err());
/// ```
#[derive(Clone, Serialize)]
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

impl<T: std::fmt::Debug, N> std::fmt::Debug for VariableList<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.vec.fmt(f)
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
        if vec.len() <= N::to_usize() {
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

    /// Creates a full list with the given element repeated.
    pub fn repeat_full(elem: T) -> Self
    where
        T: Clone,
    {
        Self {
            vec: vec![elem; N::to_usize()],
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
        N::to_usize()
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

impl<T, N: Unsigned> TryFrom<Vec<T>> for VariableList<T, N> {
    type Error = Error;

    fn try_from(vec: Vec<T>) -> Result<Self, Error> {
        Self::new(vec)
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
        let root = vec_tree_hash_root::<T>(&self.vec, N::to_usize());

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
        let n = N::to_usize();
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
        let max_len = N::to_usize();

        if bytes.is_empty() {
            return Ok(Self::default());
        }

        if T::is_ssz_fixed_len() {
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

            bytes.chunks(T::ssz_fixed_len()).try_fold(
                Vec::with_capacity(num_items),
                |mut vec, chunk| {
                    vec.push(T::from_ssz_bytes(chunk)?);
                    Ok(vec)
                },
            )
        } else {
            ssz::decode_list_of_variable_length_items(bytes, Some(max_len))
        }?
        .try_into()
        .map_err(|e| {
            ssz::DecodeError::BytesInvalid(format!("VariableList::try_from failed: {e:?}"))
        })
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
        if vec.len() <= N::to_usize() {
            Ok(VariableList {
                vec,
                _phantom: PhantomData,
            })
        } else {
            Err(serde::de::Error::custom(format!(
                "VariableList length {} exceeds maximum length {}",
                vec.len(),
                N::to_usize()
            )))
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>, N: 'static + Unsigned> arbitrary::Arbitrary<'a>
    for VariableList<T, N>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let max_size = N::to_usize();
        let rand = usize::arbitrary(u)?;
        let size = std::cmp::min(rand, max_size);
        let mut vec: Vec<T> = Vec::with_capacity(size);
        for _ in 0..size {
            vec.push(<T>::arbitrary(u)?);
        }
        Self::new(vec).map_err(|_| arbitrary::Error::IncorrectFormat)
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
    fn repeat_full() {
        let manual_list = VariableList::<u64, U5>::new(vec![42; 5]).unwrap();
        let repeat_list = VariableList::<u64, U5>::repeat_full(42);
        assert_eq!(manual_list, repeat_list);
    }

    #[test]
    fn indexing() {
        let vec = vec![1, 2];

        let mut fixed: VariableList<u64, U8192> = vec.clone().try_into().unwrap();

        assert_eq!(fixed[0], 1);
        assert_eq!(&fixed[0..1], &vec[0..1]);
        assert_eq!((fixed[..]).len(), 2);

        fixed[1] = 3;
        assert_eq!(fixed[1], 3);
    }

    #[test]
    fn length() {
        // Too long.
        let vec = vec![42; 5];
        let err = VariableList::<u64, U4>::try_from(vec.clone()).unwrap_err();
        assert_eq!(err, Error::OutOfBounds { i: 5, len: 4 });

        let vec = vec![42; 3];
        let fixed: VariableList<u64, U4> = VariableList::try_from(vec.clone()).unwrap();
        assert_eq!(&fixed[0..3], &vec[..]);
        assert_eq!(&fixed[..], &vec![42, 42, 42][..]);

        let vec = vec![];
        let fixed: VariableList<u64, U4> = VariableList::try_from(vec).unwrap();
        assert_eq!(&fixed[..], &[] as &[u64]);
    }

    #[test]
    fn deref() {
        let vec = vec![0, 2, 4, 6];
        let fixed: VariableList<u64, U4> = VariableList::try_from(vec).unwrap();

        assert_eq!(fixed.first(), Some(&0));
        assert_eq!(fixed.get(3), Some(&6));
        assert_eq!(fixed.get(4), None);
    }

    #[test]
    fn encode() {
        let vec: VariableList<u16, U2> = vec![0; 2].try_into().unwrap();
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
        round_trip::<VariableList<u16, U8>>(vec![42; 8].try_into().unwrap());
        round_trip::<VariableList<u16, U8>>(vec![0; 8].try_into().unwrap());
        round_trip::<VariableList<u16, U8>>(vec![].try_into().unwrap());
    }

    #[test]
    fn ssz_empty_list() {
        let empty_list = VariableList::<u16, U8>::default();
        let bytes = empty_list.as_ssz_bytes();
        assert!(bytes.is_empty());
        assert_eq!(VariableList::from_ssz_bytes(&[]).unwrap(), empty_list);
    }

    fn root_with_length(bytes: &[u8], len: usize) -> Hash256 {
        let root = merkle_root(bytes, 0);
        tree_hash::mix_in_length(&root, len)
    }

    #[test]
    fn tree_hash_u8() {
        let fixed: VariableList<u8, U0> = VariableList::try_from(vec![]).unwrap();
        assert_eq!(fixed.tree_hash_root(), root_with_length(&[0; 8], 0));

        for i in 0..=1 {
            let fixed: VariableList<u8, U1> = VariableList::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=8 {
            let fixed: VariableList<u8, U8> = VariableList::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=13 {
            let fixed: VariableList<u8, U13> = VariableList::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        for i in 0..=16 {
            let fixed: VariableList<u8, U16> = VariableList::try_from(vec![0; i]).unwrap();
            assert_eq!(fixed.tree_hash_root(), root_with_length(&vec![0; i], i));
        }

        let source: Vec<u8> = (0..16).collect();
        let fixed: VariableList<u8, U16> = VariableList::try_from(source.clone()).unwrap();
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

        let fixed: VariableList<A, U0> = VariableList::try_from(vec![]).unwrap();
        assert_eq!(
            fixed.tree_hash_root(),
            padded_root_with_length(&[0; 32], 0, 0),
        );

        for i in 0..=1 {
            let fixed: VariableList<A, U1> = VariableList::try_from(vec![a; i]).unwrap();
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 1),
                "U1 {}",
                i
            );
        }

        for i in 0..=8 {
            let fixed: VariableList<A, U8> = VariableList::try_from(vec![a; i]).unwrap();
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 8),
                "U8 {}",
                i
            );
        }

        for i in 0..=13 {
            let fixed: VariableList<A, U13> = VariableList::try_from(vec![a; i]).unwrap();
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 13),
                "U13 {}",
                i
            );
        }

        for i in 0..=16 {
            let fixed: VariableList<A, U16> = VariableList::try_from(vec![a; i]).unwrap();
            assert_eq!(
                fixed.tree_hash_root(),
                padded_root_with_length(&repeat(a.tree_hash_root().as_slice(), i), i, 16),
                "U16 {}",
                i
            );
        }
    }

    #[test]
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
            hint: N::to_usize() / 2,
            iter: iter.clone(),
        };

        // Don't explode.
        assert_eq!(
            List::try_from_iter(iter).unwrap(),
            List::try_from_iter(wonky_iter).unwrap()
        );
    }

    #[test]
    fn std_hash() {
        let x: VariableList<u32, U16> = VariableList::try_from(vec![3; 16]).unwrap();
        let y: VariableList<u32, U16> = VariableList::try_from(vec![4; 16]).unwrap();
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

    #[test]
    fn debug_transparent_list() {
        let list: VariableList<u64, U5> = VariableList::try_from(vec![1, 2, 3]).unwrap();
        let debug_output = format!("{:?}", list);

        assert_eq!(debug_output, "[1, 2, 3]");
    }

    // This tests the `From<Infallible>` impl for `Error`.
    #[test]
    fn error_from_infallible() {
        let result: Result<Vec<u64>, Error> =
            Vec::try_from(VariableList::<u64, U5>::repeat_full(6)).map_err(Into::into);
        assert_eq!(result, Ok(vec![6; 5]));
    }
}
