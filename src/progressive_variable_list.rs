use crate::tree_hash::progressive_vec_tree_hash_root;
use crate::variable_list::MAX_ELEMENTS_TO_PRE_ALLOCATE;
use crate::Error;
use serde::Deserialize;
use serde_derive::Serialize;
use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;
use tree_hash::Hash256;
use typenum::{Unsigned, U0};

/// Emulates a SSZ `ProgressiveList` (EIP-7916).
///
/// An ordered, heap-allocated, variable-length, homogeneous collection of `T`. This is the
/// progressive analogue of [`VariableList`](crate::VariableList). The two differ in two ways.
///
/// - Merkleization uses the progressive scheme of EIP-7916 (a right-leaning spine of binary
///   subtrees whose capacities grow by 4x), so the hash tree root is independent of any limit.
/// - The length limit `N` is optional. A non-zero limit is enforced on construction and mutation.
///
/// The type parameter `N` is a [`typenum`] unsigned integer. The default `U0` means no limit, so
/// the list can have any length. SSZ decoding rejects inputs with more than `N` elements for a
/// non-zero `N` before allocating space for them. The limit does not change the SSZ encoding or the
/// hash tree root. You can add or change it on a field without a consensus change.
///
/// Like `VariableList`, the list is backed by a Rust `Vec` and serialized identically to a plain
/// list.
///
/// Known spec divergence: encoding does not enforce the SSZ requirement that the total
/// encoding be less than 2^32 bytes. Callers must ensure this limit is respected.
/// Decoding rejects encodings at or above this limit.
///
/// ## Example
///
/// ```
/// use ssz_types::ProgressiveVariableList;
/// use ssz_types::typenum::U8;
///
/// let base: Vec<u64> = vec![1, 2, 3, 4];
///
/// // No limit (default).
/// let mut list: ProgressiveVariableList<u64> = ProgressiveVariableList::new(base.clone()).unwrap();
/// assert_eq!(&list[..], &[1, 2, 3, 4]);
///
/// // `push` succeeds as long as the optional limit is not exceeded.
/// list.push(5).unwrap();
/// assert_eq!(&list[..], &[1, 2, 3, 4, 5]);
///
/// // A limited list rejects oversized input.
/// type Bounded = ProgressiveVariableList<u64, U8>;
/// assert_eq!(Bounded::max_len(), Some(8));
/// assert!(Bounded::new(vec![0; 9]).is_err());
/// ```
#[derive(Clone, Serialize)]
#[serde(transparent)]
pub struct ProgressiveVariableList<T, N = U0> {
    vec: Vec<T>,
    #[serde(skip)]
    _phantom: PhantomData<N>,
}

impl<T: PartialEq, N> PartialEq for ProgressiveVariableList<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}
impl<T: Eq, N> Eq for ProgressiveVariableList<T, N> {}
impl<T: std::hash::Hash, N> std::hash::Hash for ProgressiveVariableList<T, N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.hash(state);
    }
}

impl<T: std::fmt::Debug, N> std::fmt::Debug for ProgressiveVariableList<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.vec.fmt(f)
    }
}

impl<T, N> ProgressiveVariableList<T, N> {
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
        self.vec.is_empty()
    }

    /// Returns the contents as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.vec
    }

    /// Consumes `self`, returning the underlying `Vec`.
    pub fn into_vec(self) -> Vec<T> {
        self.vec
    }
}

impl<T, N: Unsigned> ProgressiveVariableList<T, N> {
    /// Create a list from a `Vec`, returning an error if it exceeds the optional limit.
    pub fn new(vec: Vec<T>) -> Result<Self, Error> {
        if let Some(max) = Self::max_len() {
            if vec.len() > max {
                return Err(Error::OutOfBounds {
                    i: vec.len(),
                    len: max,
                });
            }
        }
        Ok(Self {
            vec,
            _phantom: PhantomData,
        })
    }

    /// Appends `value` to the back of `self`, returning an error if it would exceed the optional limit.
    pub fn push(&mut self, value: T) -> Result<(), Error> {
        if let Some(max) = Self::max_len() {
            if self.vec.len() >= max {
                return Err(Error::OutOfBounds {
                    i: self.vec.len() + 1,
                    len: max,
                });
            }
        }
        self.vec.push(value);
        Ok(())
    }

    /// Returns the optional length limit.
    ///
    /// `None` means no limit (`N = U0`). `Some(n)` rejects any input with more than `n` elements.
    pub fn max_len() -> Option<usize> {
        match N::to_usize() {
            0 => None,
            n => Some(n),
        }
    }

    /// The size hint comes from untrusted input, so the limit and a constant cap the reservation.
    fn capacity_to_reserve((lower, upper): (usize, Option<usize>)) -> usize {
        let cap = Self::max_len().map_or(MAX_ELEMENTS_TO_PRE_ALLOCATE, |max| {
            max.min(MAX_ELEMENTS_TO_PRE_ALLOCATE)
        });
        upper.unwrap_or(lower).min(cap)
    }
}

impl<T, N: Unsigned> TryFrom<Vec<T>> for ProgressiveVariableList<T, N> {
    type Error = Error;

    fn try_from(vec: Vec<T>) -> Result<Self, Error> {
        Self::new(vec)
    }
}

impl<T, N> From<ProgressiveVariableList<T, N>> for Vec<T> {
    fn from(list: ProgressiveVariableList<T, N>) -> Vec<T> {
        list.vec
    }
}

impl<T, N> Default for ProgressiveVariableList<T, N> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T, N, I: SliceIndex<[T]>> Index<I> for ProgressiveVariableList<T, N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, N, I: SliceIndex<[T]>> IndexMut<I> for ProgressiveVariableList<T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T, N> Deref for ProgressiveVariableList<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<T, N> DerefMut for ProgressiveVariableList<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<T, N> AsRef<[T]> for ProgressiveVariableList<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<'a, T, N> IntoIterator for &'a ProgressiveVariableList<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, N> IntoIterator for ProgressiveVariableList<T, N> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T, N> tree_hash::TreeHash for ProgressiveVariableList<T, N>
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
        let root = progressive_vec_tree_hash_root::<T>(&self.vec);

        tree_hash::mix_in_length(&root, self.len())
    }
}

impl<T, N> ssz::Encode for ProgressiveVariableList<T, N>
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

impl<T, N: Unsigned> ssz::TryFromIter<T> for ProgressiveVariableList<T, N> {
    type Error = Error;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = T>,
    {
        let iter = value.into_iter();
        let capacity = Self::capacity_to_reserve(iter.size_hint());
        let mut list = Self::new(Vec::with_capacity(capacity))?;
        for item in iter {
            list.push(item)?;
        }
        Ok(list)
    }
}

impl<T, N> ssz::Decode for ProgressiveVariableList<T, N>
where
    T: ssz::Decode + 'static,
    N: Unsigned,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.len() > ssz::MAX_LENGTH_VALUE {
            return Err(ssz::DecodeError::BytesInvalid(
                "ProgressiveVariableList SSZ encoding must be less than 2^32 bytes".into(),
            ));
        }

        if bytes.is_empty() {
            return Ok(Self::default());
        }

        let max_len = Self::max_len();

        if TypeId::of::<T>() == TypeId::of::<u8>() {
            if let Some(max) = max_len {
                if bytes.len() > max {
                    return Err(ssz::DecodeError::BytesInvalid(format!(
                        "ProgressiveVariableList of {} items exceeds maximum of {}",
                        bytes.len(),
                        max
                    )));
                }
            }
            return Self::new(crate::u8_bytes_to_vec(bytes))
                .map_err(|e| ssz::DecodeError::BytesInvalid(e.to_string()));
        }

        if T::is_ssz_fixed_len() {
            let item_len = T::ssz_fixed_len();
            // A zero-length item is a distinct error, matching `VariableList::from_ssz_bytes`.
            // It also guards the `chunks_exact` below against a zero divisor.
            let num_items = bytes
                .len()
                .checked_div(item_len)
                .ok_or(ssz::DecodeError::ZeroLengthItem)?;

            if let Some(max) = max_len {
                if num_items > max {
                    return Err(ssz::DecodeError::BytesInvalid(format!(
                        "ProgressiveVariableList of {} items exceeds maximum of {}",
                        num_items, max
                    )));
                }
            }

            if !bytes.len().is_multiple_of(item_len) {
                return Err(ssz::DecodeError::BytesInvalid(format!(
                    "ProgressiveVariableList has {} bytes, not a multiple of item length {}",
                    bytes.len(),
                    item_len
                )));
            }

            let mut vec = Vec::with_capacity(num_items);
            for chunk in bytes.chunks_exact(item_len) {
                vec.push(T::from_ssz_bytes(chunk)?);
            }
            Self::new(vec).map_err(|e| ssz::DecodeError::BytesInvalid(e.to_string()))
        } else {
            ssz::decode_list_of_variable_length_items(bytes, max_len)
        }
    }
}

impl<'de, T, N> Deserialize<'de> for ProgressiveVariableList<T, N>
where
    T: Deserialize<'de>,
    N: Unsigned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<T>::deserialize(deserializer)?;
        Self::new(vec).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>, N: Unsigned> arbitrary::Arbitrary<'a>
    for ProgressiveVariableList<T, N>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut vec = <Vec<T>>::arbitrary(u)?;
        if let Some(max) = Self::max_len() {
            vec.truncate(max);
        }
        Self::new(vec).map_err(|_| arbitrary::Error::IncorrectFormat)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <Vec<T>>::size_hint(depth)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ssz::{Decode, Encode, TryFromIter};
    use tree_hash::TreeHash;
    use typenum::{U256, U4};

    #[test]
    fn new_and_push_unbounded() {
        let mut list: ProgressiveVariableList<u64> =
            ProgressiveVariableList::new(vec![1, 2, 3]).unwrap();
        assert_eq!(&list[..], &[1, 2, 3]);
        list.push(4).unwrap();
        assert_eq!(&list[..], &[1, 2, 3, 4]);
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn limit_bounds_construction() {
        type Bounded = ProgressiveVariableList<u64, U4>;
        for len in [0, 3, 4] {
            let values = vec![1; len];
            assert_eq!(Bounded::new(values.clone()).unwrap().as_slice(), values);
            assert_eq!(
                Bounded::try_from(values.clone()).unwrap().as_slice(),
                values
            );
            assert_eq!(
                Bounded::try_from_iter(values.clone()).unwrap().as_slice(),
                values
            );
        }
        let err = Error::OutOfBounds { i: 5, len: 4 };
        assert_eq!(Bounded::new(vec![1; 5]), Err(err.clone()));
        assert_eq!(Bounded::try_from(vec![1; 5]), Err(err.clone()));
        let iter = (0..5).chain(std::iter::once_with(|| panic!("iterated past the limit")));
        assert_eq!(Bounded::try_from_iter(iter), Err(err));
        assert_eq!(
            ProgressiveVariableList::<u64>::try_from_iter(0..5)
                .unwrap()
                .len(),
            5
        );
    }

    #[test]
    fn limit_bounds_push() {
        let mut list = ProgressiveVariableList::<u64, U4>::new(vec![1, 2, 3]).unwrap();
        list.push(4).unwrap();
        assert_eq!(list.push(5), Err(Error::OutOfBounds { i: 5, len: 4 }));
        assert_eq!(&list[..], &[1, 2, 3, 4]);
    }

    /// Yields the items of a `Vec` but claims a size hint of `usize::MAX`.
    struct LyingIter(std::vec::IntoIter<u64>);

    impl Iterator for LyingIter {
        type Item = u64;

        fn next(&mut self) -> Option<u64> {
            self.0.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (usize::MAX, Some(usize::MAX))
        }
    }

    #[test]
    fn try_from_iter_reserves_the_exact_size_hint() {
        let list = ProgressiveVariableList::<u64>::try_from_iter(0..100).unwrap();
        assert_eq!(list.vec.capacity(), 100);
    }

    #[test]
    fn try_from_iter_caps_a_lying_size_hint_at_the_limit() {
        let iter = LyingIter(vec![1, 2, 3].into_iter());
        let list = ProgressiveVariableList::<u64, U4>::try_from_iter(iter).unwrap();
        assert_eq!(&list[..], &[1, 2, 3]);
        assert_eq!(list.vec.capacity(), 4);
    }

    #[test]
    fn try_from_iter_caps_a_lying_size_hint_without_a_limit() {
        let iter = LyingIter(vec![1, 2, 3].into_iter());
        let list = ProgressiveVariableList::<u64>::try_from_iter(iter).unwrap();
        assert_eq!(&list[..], &[1, 2, 3]);
        assert_eq!(list.vec.capacity(), MAX_ELEMENTS_TO_PRE_ALLOCATE);
    }

    #[cfg(feature = "arbitrary")]
    #[test]
    fn limit_bounds_arbitrary() {
        use arbitrary::{Arbitrary, Unstructured};

        let data = [1; 32];
        let unbounded =
            ProgressiveVariableList::<u8>::arbitrary(&mut Unstructured::new(&data)).unwrap();
        let bounded =
            ProgressiveVariableList::<u8, U4>::arbitrary(&mut Unstructured::new(&data)).unwrap();
        assert!(unbounded.len() > 4);
        assert_eq!(bounded.as_slice(), &unbounded[..4]);
    }

    fn ssz_round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn ssz_round_trip_bytes() {
        ssz_round_trip::<ProgressiveVariableList<u8>>(
            ProgressiveVariableList::new(vec![]).unwrap(),
        );
        ssz_round_trip::<ProgressiveVariableList<u8>>(
            ProgressiveVariableList::new(vec![42; 100]).unwrap(),
        );
        // Serializes identically to a plain byte-list.
        let bytes = ProgressiveVariableList::<u8>::new(vec![1, 2, 3]).unwrap();
        assert_eq!(bytes.as_ssz_bytes(), vec![1, 2, 3]);
    }

    #[test]
    fn ssz_round_trip_u64() {
        ssz_round_trip::<ProgressiveVariableList<u64>>(
            ProgressiveVariableList::new(vec![42; 9]).unwrap(),
        );
    }

    #[test]
    fn max_len_reports_optional_limit() {
        assert_eq!(ProgressiveVariableList::<u64>::max_len(), None);
        assert_eq!(ProgressiveVariableList::<u64, U4>::max_len(), Some(4));
    }

    #[test]
    fn limit_bounds_fixed_len_ssz_decode() {
        let ok = ProgressiveVariableList::<u64>::new(vec![1, 2, 3, 4])
            .unwrap()
            .as_ssz_bytes();
        assert!(ProgressiveVariableList::<u64, U4>::from_ssz_bytes(&ok).is_ok());

        let too_many = ProgressiveVariableList::<u64>::new(vec![1, 2, 3, 4, 5])
            .unwrap()
            .as_ssz_bytes();
        assert!(ProgressiveVariableList::<u64, U4>::from_ssz_bytes(&too_many).is_err());

        assert!(ProgressiveVariableList::<u64>::from_ssz_bytes(&too_many).is_ok());
    }

    #[test]
    fn limit_bounds_variable_len_ssz_decode() {
        type Inner = ProgressiveVariableList<u8>;
        let items: Vec<Inner> = (0..5)
            .map(|_| ProgressiveVariableList::new(vec![1]).unwrap())
            .collect();
        let encoded = ProgressiveVariableList::<Inner>::new(items)
            .unwrap()
            .as_ssz_bytes();

        assert!(ProgressiveVariableList::<Inner, U4>::from_ssz_bytes(&encoded).is_err());
        assert!(ProgressiveVariableList::<Inner>::from_ssz_bytes(&encoded).is_ok());
    }

    #[test]
    fn limit_bounds_byte_list_ssz_decode() {
        let encoded = ProgressiveVariableList::<u8>::new(vec![0; 5])
            .unwrap()
            .as_ssz_bytes();
        assert!(ProgressiveVariableList::<u8, U4>::from_ssz_bytes(&encoded).is_err());
        assert!(ProgressiveVariableList::<u8>::from_ssz_bytes(&encoded).is_ok());
    }

    #[test]
    fn limit_bounds_json_deserialize() {
        let json = "[1,2,3,4,5]";
        assert!(serde_json::from_str::<ProgressiveVariableList<u64, U4>>(json).is_err());
        assert!(serde_json::from_str::<ProgressiveVariableList<u64>>(json).is_ok());
    }

    #[test]
    fn limit_does_not_change_encoding_or_root() {
        let values = vec![9u64, 8, 7];
        let unbounded = ProgressiveVariableList::<u64>::new(values.clone()).unwrap();
        let bounded = ProgressiveVariableList::<u64, U256>::new(values).unwrap();
        assert_eq!(unbounded.as_ssz_bytes(), bounded.as_ssz_bytes());
        assert_eq!(unbounded.tree_hash_root(), bounded.tree_hash_root());
    }

    #[test]
    fn serde_is_a_sequence() {
        // Matches `VariableList`: the default serde representation is a JSON sequence, not hex.
        let list: ProgressiveVariableList<u8> =
            ProgressiveVariableList::new(vec![1, 2, 255]).unwrap();
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(json, "[1,2,255]");
        assert_eq!(
            serde_json::from_str::<ProgressiveVariableList<u8>>(&json).unwrap(),
            list
        );
    }

    #[test]
    fn tree_hash_byte_list() {
        use crate::VariableList;
        use tree_hash::mix_in_length;

        // Empty list: mix_in_length(ZERO, 0). (Exact progressive-hasher math is covered by the
        // `tree_hash` crate's own tests and the EF spec vectors.)
        assert_eq!(
            ProgressiveVariableList::<u8>::empty().tree_hash_root(),
            mix_in_length(&Hash256::ZERO, 0)
        );

        // Deterministic and non-zero for a non-empty list.
        let bytes: Vec<u8> = (0..40).collect();
        let root = ProgressiveVariableList::<u8>::new(bytes.clone())
            .unwrap()
            .tree_hash_root();
        assert_eq!(
            root,
            ProgressiveVariableList::<u8>::new(bytes.clone())
                .unwrap()
                .tree_hash_root()
        );
        assert_ne!(root, mix_in_length(&Hash256::ZERO, bytes.len()));

        // Progressive merkleization differs from the fixed-depth `VariableList` root: same data,
        // different scheme.
        let bounded = VariableList::<u8, U256>::new(bytes).unwrap();
        assert_ne!(root, bounded.tree_hash_root());
    }
}
