use crate::typenum_helpers::to_usize;
use tree_hash::{Hash256, MerkleHasher, TreeHash, TreeHashType};
use typenum::Unsigned;

pub fn packing_factor<T: TreeHash>() -> usize {
    match T::tree_hash_type() {
        TreeHashType::Basic => T::tree_hash_packing_factor(),
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => 1,
    }
}

mod default_impl {
    use super::*;
    /// A helper function providing common functionality between the `TreeHash` implementations for
    /// `FixedVector` and `VariableList`.
    pub fn vec_tree_hash_root<T, N>(vec: &[T]) -> Hash256
    where
        T: TreeHash,
        N: Unsigned,
    {
        match T::tree_hash_type() {
            TreeHashType::Basic => {
                let mut hasher = MerkleHasher::with_leaves(
                    (to_usize::<N>() + T::tree_hash_packing_factor() - 1)
                        / T::tree_hash_packing_factor(),
                );

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
                let mut hasher = MerkleHasher::with_leaves(N::to_usize());

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
}

#[cfg(feature = "cap-typenum-to-usize-overflow")]
mod arch_32x_workaround {
    use super::*;
    use ethereum_hashing::{hash32_concat, ZERO_HASHES};
    use tree_hash::{Hash256, TreeHash};
    use typenum::Unsigned;

    type MaxDepth = typenum::U536870912;

    fn pad_to_depth<Current: Unsigned, Target: Unsigned>(
        hash: Hash256,
        target_depth: usize,
        current_depth: usize,
    ) -> Hash256 {
        let mut curhash: [u8; 32] = hash.0;
        for depth in current_depth..target_depth {
            curhash = hash32_concat(&curhash, ZERO_HASHES[depth].as_slice());
        }
        curhash.into()
    }

    fn target_tree_depth<T: TreeHash, N: Unsigned>() -> usize {
        let packing_factor = packing_factor::<T>();
        let packing_factor_log2 = packing_factor.next_power_of_two().ilog2() as usize;
        let tree_depth = N::to_u64().next_power_of_two().ilog2() as usize;
        tree_depth - packing_factor_log2
    }

    pub fn vec_tree_hash_root<T: TreeHash, N: Unsigned>(vec: &[T]) -> Hash256 {
        if N::to_u64() <= MaxDepth::to_u64() {
            default_impl::vec_tree_hash_root::<T, N>(vec)
        } else {
            let main_tree_hash = default_impl::vec_tree_hash_root::<T, MaxDepth>(vec);

            let target_depth = target_tree_depth::<T, N>();
            let current_depth = target_tree_depth::<T, MaxDepth>();

            pad_to_depth::<MaxDepth, N>(main_tree_hash, target_depth, current_depth)
        }
    }
}

#[cfg(any(
    target_pointer_width = "64",
    not(feature = "cap-typenum-to-usize-overflow")
))]
pub use default_impl::vec_tree_hash_root;

#[cfg(all(
    not(target_pointer_width = "64"),
    feature = "cap-typenum-to-usize-overflow"
))]
pub use arch_32x_workaround::vec_tree_hash_root;
