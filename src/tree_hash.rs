use tree_hash::{Hash256, MerkleHasher, ProgressiveMerkleHasher, TreeHash, TreeHashType};

/// A helper function providing common functionality between the `TreeHash` implementations for
/// `FixedVector` and `VariableList`.
pub fn vec_tree_hash_root<T>(vec: &[T], max_leaves: usize) -> Hash256
where
    T: TreeHash,
{
    match T::tree_hash_type() {
        TreeHashType::Basic => {
            let mut hasher =
                MerkleHasher::with_leaves(max_leaves.div_ceil(T::tree_hash_packing_factor()));

            for item in vec {
                hasher
                    .write(&item.tree_hash_packed_encoding())
                    .expect("ssz_types variable vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz_types variable vec should not have a remaining buffer")
        }
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => {
            let mut hasher = MerkleHasher::with_leaves(max_leaves);

            for item in vec {
                hasher
                    .write(item.tree_hash_root().as_slice())
                    .expect("ssz_types vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz_types vec should not have a remaining buffer")
        }
    }
}

/// Like `vec_tree_hash_root`, but uses progressive merkleization from EIP-7916.
pub fn progressive_vec_tree_hash_root<T>(vec: &[T]) -> Hash256
where
    T: TreeHash,
{
    let mut hasher = ProgressiveMerkleHasher::new();

    match T::tree_hash_type() {
        TreeHashType::Basic => {
            for item in vec {
                hasher
                    .write(&item.tree_hash_packed_encoding())
                    .expect("ssz_types progressive vec should not exceed the leaf limit");
            }
        }
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => {
            for item in vec {
                hasher
                    .write(item.tree_hash_root().as_slice())
                    .expect("ssz_types progressive vec should not exceed the leaf limit");
            }
        }
    }

    hasher
        .finish()
        .expect("ssz_types progressive vec should not have a remaining buffer")
}
