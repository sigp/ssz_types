use crate::VariableList;
use tree_hash::prototype::{get_vector_item_position, vector_chunk_count, Resolve, VecIndex};
use tree_hash::{Hash256, MerkleHasher, TreeHash, TreeHashType};
use typenum::{
    generic_const_mappings::{Const, ToUInt, U},
    Unsigned,
};

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
                (N::to_usize() + T::tree_hash_packing_factor() - 1) / T::tree_hash_packing_factor(),
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

impl<T, const I: usize, const N: usize> Resolve<VecIndex<I, N>> for VariableList<T, U<N>>
where
    T: TreeHash,
    Const<N>: ToUInt,
{
    type Output = T;

    fn gindex(parent_index: usize) -> usize {
        // Base index is 2 due to length mixin.
        let base_index = 2;

        // Chunk count takes into account packing of leaves.
        let chunk_count = vector_chunk_count::<T>(N);

        let pos = get_vector_item_position::<T>(I);

        // Gindex of Nth element of this vector.
        parent_index * base_index * chunk_count.next_power_of_two() + pos
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tree_hash::prototype::{Field, Path, Resolve, VecIndex};
    use tree_hash_derive::TreeHash;
    use typenum::{U10, U5};

    // Some example structs.
    #[derive(TreeHash)]
    struct Nested3 {
        x3: Nested2,
        y3: Nested1,
    }

    #[derive(TreeHash)]
    struct Nested2 {
        x2: Nested1,
        y2: Nested1,
    }

    #[derive(TreeHash)]
    struct Nested1 {
        x1: u64,
        y1: VariableList<u64, U10>,
    }

    // Fields of Nested3 (these would be generated).
    struct FieldX3;
    struct FieldY3;

    impl Field for FieldX3 {
        const NUM_FIELDS: usize = 2;
        const INDEX: usize = 0;
    }

    impl Field for FieldY3 {
        const NUM_FIELDS: usize = 2;
        const INDEX: usize = 1;
    }

    // Fields of Nested2 (generated).
    struct FieldX2;
    struct FieldY2;

    impl Field for FieldX2 {
        const NUM_FIELDS: usize = 2;
        const INDEX: usize = 0;
    }

    impl Field for FieldY2 {
        const NUM_FIELDS: usize = 2;
        const INDEX: usize = 1;
    }

    // Fields of Nested1 (generated).
    struct FieldX1;
    struct FieldY1;

    impl Field for FieldX1 {
        const NUM_FIELDS: usize = 2;
        const INDEX: usize = 0;
    }

    impl Field for FieldY1 {
        const NUM_FIELDS: usize = 2;
        const INDEX: usize = 1;
    }

    // Implementations of Resolve (generated).
    impl Resolve<FieldX3> for Nested3 {
        type Output = Nested2;

        fn gindex(parent_index: usize) -> usize {
            parent_index * <FieldX3 as Field>::NUM_FIELDS.next_power_of_two()
                + <FieldX3 as Field>::INDEX
        }
    }

    impl Resolve<FieldY3> for Nested3 {
        type Output = Nested1;

        fn gindex(parent_index: usize) -> usize {
            parent_index * <FieldY3 as Field>::NUM_FIELDS.next_power_of_two()
                + <FieldY3 as Field>::INDEX
        }
    }

    impl Resolve<FieldX2> for Nested2 {
        type Output = Nested1;

        fn gindex(parent_index: usize) -> usize {
            parent_index * <FieldX2 as Field>::NUM_FIELDS.next_power_of_two()
                + <FieldX2 as Field>::INDEX
        }
    }

    impl Resolve<FieldY2> for Nested2 {
        type Output = Nested1;

        fn gindex(parent_index: usize) -> usize {
            parent_index * <FieldY2 as Field>::NUM_FIELDS.next_power_of_two()
                + <FieldY2 as Field>::INDEX
        }
    }

    impl Resolve<FieldX1> for Nested1 {
        type Output = u64;

        fn gindex(parent_index: usize) -> usize {
            parent_index * <FieldX1 as Field>::NUM_FIELDS.next_power_of_two()
                + <FieldX1 as Field>::INDEX
        }
    }

    impl Resolve<FieldY1> for Nested1 {
        type Output = VariableList<u64, U10>;

        fn gindex(parent_index: usize) -> usize {
            parent_index * <FieldY1 as Field>::NUM_FIELDS.next_power_of_two()
                + <FieldY1 as Field>::INDEX
        }
    }

    // x3.x2.x1
    type FieldX3X2X1 = Path<FieldX3, Path<FieldX2, FieldX1>>;

    // x3.x2.x1
    type FieldX3X2Y1 = Path<FieldX3, Path<FieldX2, FieldY1>>;

    // x3.y2.y1.5
    type FieldX3Y2Y1I5 = Path<FieldX3, Path<FieldY2, Path<FieldY1, VecIndex<5, 10>>>>;

    // 0.x3.y2.y1.5
    type FieldI0X3Y2Y1I5 =
        Path<VecIndex<0, 5>, Path<FieldX3, Path<FieldY2, Path<FieldY1, VecIndex<5, 10>>>>>;

    // This evaluates to u64 at compile-time.
    type TypeOfFieldX3X2X1 = <Nested3 as Resolve<FieldX3X2X1>>::Output;

    #[test]
    fn gindex_basics() {
        // This works but just shows compile-time field resolution.
        let x: TypeOfFieldX3X2X1 = 0u64;

        // Gindex computation.
        assert_eq!(<Nested3 as Resolve<FieldX3X2X1>>::gindex(1), 8);
        assert_eq!(<Nested3 as Resolve<FieldX3X2Y1>>::gindex(1), 9);

        // FIXME: Not sure if these values are correct
        assert_eq!(<Nested3 as Resolve<FieldX3Y2Y1I5>>::gindex(1), 89);
        assert_eq!(
            <VariableList<Nested3, U5> as Resolve<FieldI0X3Y2Y1I5>>::gindex(1),
            1049
        );
    }
}
