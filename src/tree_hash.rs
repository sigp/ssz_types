use tree_hash::{Hash256, MerkleHasher, ProgressiveMerkleHasher, TreeHash, TreeHashType};

/// A hasher that accepts leaf chunks one at a time and produces a root.
///
/// Lets `hash_vec` drive either the fixed-depth `MerkleHasher` or the progressive
/// `ProgressiveMerkleHasher` with one shared loop.
trait VecHasher: Sized {
    type Error: std::fmt::Debug;

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
    fn finish(self) -> Result<Hash256, Self::Error>;
}

impl VecHasher for MerkleHasher {
    type Error = tree_hash::Error;

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        MerkleHasher::write(self, bytes)
    }
    fn finish(self) -> Result<Hash256, Self::Error> {
        MerkleHasher::finish(self)
    }
}

impl VecHasher for ProgressiveMerkleHasher {
    type Error = tree_hash::ProgressiveMerkleHasherError;

    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        ProgressiveMerkleHasher::write(self, bytes)
    }
    fn finish(self) -> Result<Hash256, Self::Error> {
        ProgressiveMerkleHasher::finish(self)
    }
}

/// Feed `vec` into `hasher` (packed encodings for basic types, chunk roots otherwise) and finish.
fn hash_vec<T, H>(mut hasher: H, vec: &[T]) -> Hash256
where
    T: TreeHash,
    H: VecHasher,
{
    match T::tree_hash_type() {
        TreeHashType::Basic => {
            for item in vec {
                hasher
                    .write(&item.tree_hash_packed_encoding())
                    .expect("ssz_types vec should not contain more elements than max");
            }
        }
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => {
            for item in vec {
                hasher
                    .write(item.tree_hash_root().as_slice())
                    .expect("ssz_types vec should not contain more elements than max");
            }
        }
    }

    hasher
        .finish()
        .expect("ssz_types vec should not have a remaining buffer")
}

/// A helper function providing common functionality between the `TreeHash` implementations for
/// `FixedVector` and `VariableList`.
pub fn vec_tree_hash_root<T>(vec: &[T], max_leaves: usize) -> Hash256
where
    T: TreeHash,
{
    let leaves = match T::tree_hash_type() {
        TreeHashType::Basic => max_leaves.div_ceil(T::tree_hash_packing_factor()),
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => max_leaves,
    };
    hash_vec(MerkleHasher::with_leaves(leaves), vec)
}

/// Like `vec_tree_hash_root`, but uses progressive merkleization from EIP-7916.
pub fn progressive_vec_tree_hash_root<T>(vec: &[T]) -> Hash256
where
    T: TreeHash,
{
    hash_vec(ProgressiveMerkleHasher::new(), vec)
}
