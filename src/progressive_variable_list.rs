use crate::tree_hash::progressive_vec_tree_hash_root;
use serde::Deserialize;
use serde_derive::Serialize;
use std::any::TypeId;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::slice::SliceIndex;
use tree_hash::Hash256;

/// Emulates a SSZ `ProgressiveList` (EIP-7916).
///
/// An ordered, heap-allocated, variable-length, homogeneous collection of `T` with **no** capacity
/// limit. This is the progressive analogue of [`VariableList`](crate::VariableList): the two are
/// identical except that
///
/// - there is no type-level maximum length (`N`), so construction and `push` are infallible, and
/// - merkleization uses the progressive scheme of EIP-7916 (a right-leaning spine of binary
///   subtrees whose capacities grow by 4x), so the hash tree root is independent of any limit.
///
/// Like `VariableList`, it is backed by a Rust `Vec` and serialized identically to a plain list.
///
/// ## Example
///
/// ```
/// use ssz_types::ProgressiveVariableList;
///
/// let base: Vec<u64> = vec![1, 2, 3, 4];
///
/// let mut list: ProgressiveVariableList<u64> = ProgressiveVariableList::new(base.clone());
/// assert_eq!(&list[..], &[1, 2, 3, 4]);
///
/// // Unlike `VariableList`, `push` cannot fail.
/// list.push(5);
/// assert_eq!(&list[..], &[1, 2, 3, 4, 5]);
/// ```
#[derive(Clone, Serialize)]
#[serde(transparent)]
pub struct ProgressiveVariableList<T> {
    vec: Vec<T>,
}

impl<T: PartialEq> PartialEq for ProgressiveVariableList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}
impl<T: Eq> Eq for ProgressiveVariableList<T> {}
impl<T: std::hash::Hash> std::hash::Hash for ProgressiveVariableList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vec.hash(state);
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ProgressiveVariableList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.vec.fmt(f)
    }
}

impl<T> ProgressiveVariableList<T> {
    /// Create a list from a `Vec`. Infallible, as there is no capacity limit.
    pub fn new(vec: Vec<T>) -> Self {
        Self { vec }
    }

    /// Create an empty list.
    pub fn empty() -> Self {
        Self { vec: vec![] }
    }

    /// Returns the number of values presently in `self`.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// True if `self` does not contain any values.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Appends `value` to the back of `self`. Infallible, as there is no capacity limit.
    pub fn push(&mut self, value: T) {
        self.vec.push(value);
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

impl<T> From<Vec<T>> for ProgressiveVariableList<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::new(vec)
    }
}

impl<T> From<ProgressiveVariableList<T>> for Vec<T> {
    fn from(list: ProgressiveVariableList<T>) -> Vec<T> {
        list.vec
    }
}

impl<T> Default for ProgressiveVariableList<T> {
    fn default() -> Self {
        Self {
            vec: Vec::default(),
        }
    }
}

impl<T> FromIterator<T> for ProgressiveVariableList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for ProgressiveVariableList<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.vec, index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for ProgressiveVariableList<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.vec, index)
    }
}

impl<T> Deref for ProgressiveVariableList<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<T> DerefMut for ProgressiveVariableList<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.vec[..]
    }
}

impl<T> AsRef<[T]> for ProgressiveVariableList<T> {
    fn as_ref(&self) -> &[T] {
        &self.vec[..]
    }
}

impl<'a, T> IntoIterator for &'a ProgressiveVariableList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for ProgressiveVariableList<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<T> tree_hash::TreeHash for ProgressiveVariableList<T>
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

impl<T> ssz::Encode for ProgressiveVariableList<T>
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

impl<T> ssz::TryFromIter<T> for ProgressiveVariableList<T> {
    type Error = std::convert::Infallible;

    fn try_from_iter<I>(value: I) -> Result<Self, Self::Error>
    where
        I: IntoIterator<Item = T>,
    {
        Ok(Self::new(value.into_iter().collect()))
    }
}

impl<T> ssz::Decode for ProgressiveVariableList<T>
where
    T: ssz::Decode + 'static,
{
    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }

        if TypeId::of::<T>() == TypeId::of::<u8>() {
            return Ok(Self::new(crate::u8_bytes_to_vec(bytes)));
        }

        if T::is_ssz_fixed_len() {
            let item_len = T::ssz_fixed_len();
            // A zero-length item is a distinct error, matching `VariableList::from_ssz_bytes`.
            // It also guards the `chunks_exact` below against a zero divisor.
            bytes
                .len()
                .checked_div(item_len)
                .ok_or(ssz::DecodeError::ZeroLengthItem)?;

            if !bytes.len().is_multiple_of(item_len) {
                return Err(ssz::DecodeError::BytesInvalid(format!(
                    "ProgressiveVariableList has {} bytes, not a multiple of item length {}",
                    bytes.len(),
                    item_len
                )));
            }

            bytes
                .chunks_exact(item_len)
                .map(T::from_ssz_bytes)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::new)
        } else {
            ssz::decode_list_of_variable_length_items(bytes, None).map(Self::new)
        }
    }
}

impl<'de, T> Deserialize<'de> for ProgressiveVariableList<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(Vec::<T>::deserialize(deserializer)?))
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for ProgressiveVariableList<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::new(<Vec<T>>::arbitrary(u)?))
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <Vec<T>>::size_hint(depth)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ssz::{Decode, Encode};
    use tree_hash::TreeHash;

    #[test]
    fn new_and_push_infallible() {
        let mut list: ProgressiveVariableList<u64> = ProgressiveVariableList::new(vec![1, 2, 3]);
        assert_eq!(&list[..], &[1, 2, 3]);
        list.push(4);
        assert_eq!(&list[..], &[1, 2, 3, 4]);
        assert_eq!(list.len(), 4);
    }

    fn ssz_round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(item: T) {
        let encoded = &item.as_ssz_bytes();
        assert_eq!(item.ssz_bytes_len(), encoded.len());
        assert_eq!(T::from_ssz_bytes(encoded), Ok(item));
    }

    #[test]
    fn ssz_round_trip_bytes() {
        ssz_round_trip::<ProgressiveVariableList<u8>>(ProgressiveVariableList::new(vec![]));
        ssz_round_trip::<ProgressiveVariableList<u8>>(ProgressiveVariableList::new(vec![42; 100]));
        // Serializes identically to a plain byte-list.
        let bytes = ProgressiveVariableList::<u8>::new(vec![1, 2, 3]);
        assert_eq!(bytes.as_ssz_bytes(), vec![1, 2, 3]);
    }

    #[test]
    fn ssz_round_trip_u64() {
        ssz_round_trip::<ProgressiveVariableList<u64>>(ProgressiveVariableList::new(vec![42; 9]));
    }

    #[test]
    fn serde_is_a_sequence() {
        // Matches `VariableList`: the default serde representation is a JSON sequence, not hex.
        let list: ProgressiveVariableList<u8> = ProgressiveVariableList::new(vec![1, 2, 255]);
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
        use typenum::U256;

        // Empty list: mix_in_length(ZERO, 0). (Exact progressive-hasher math is covered by the
        // `tree_hash` crate's own tests and the EF spec vectors.)
        assert_eq!(
            ProgressiveVariableList::<u8>::empty().tree_hash_root(),
            mix_in_length(&Hash256::ZERO, 0)
        );

        // Deterministic and non-zero for a non-empty list.
        let bytes: Vec<u8> = (0..40).collect();
        let root = ProgressiveVariableList::new(bytes.clone()).tree_hash_root();
        assert_eq!(
            root,
            ProgressiveVariableList::new(bytes.clone()).tree_hash_root()
        );
        assert_ne!(root, mix_in_length(&Hash256::ZERO, bytes.len()));

        // Progressive merkleization differs from the fixed-depth `VariableList` root: same data,
        // different scheme.
        let bounded = VariableList::<u8, U256>::new(bytes).unwrap();
        assert_ne!(root, bounded.tree_hash_root());
    }
}
