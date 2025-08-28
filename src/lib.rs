//! Provides types with unique properties required for SSZ serialization and Merklization:
//!
//! - `FixedVector`: A heap-allocated list with a size that is fixed at compile time.
//! - `VariableList`: A heap-allocated list that cannot grow past a type-level maximum length.
//! - `BitList`: A heap-allocated bitfield that with a type-level _maximum_ length.
//! - `BitVector`: A heap-allocated bitfield that with a type-level _fixed__ length.
//!
//! These structs are required as SSZ serialization and Merklization rely upon type-level lengths
//! for padding and verification.
//!
//! Adheres to the Ethereum 2.0 [SSZ
//! specification](https://github.com/ethereum/eth2.0-specs/blob/v0.12.1/ssz/simple-serialize.md)
//! at v0.12.1.
//!
//! ## Example
//! ```
//! use ssz_types::*;
//!
//! pub struct Example {
//!     bit_vector: BitVector<typenum::U8>,
//!     bit_list: BitList<typenum::U8>,
//!     variable_list: VariableList<u64, typenum::U8>,
//!     fixed_vector: FixedVector<u64, typenum::U8>,
//! }
//!
//! let mut example = Example {
//!     bit_vector: Bitfield::new(),
//!     bit_list: Bitfield::with_capacity(4).unwrap(),
//!     variable_list: VariableList::try_from(vec![0, 1]).unwrap(),
//!     fixed_vector: FixedVector::try_from(vec![2, 3, 4, 5, 6, 7, 8, 9]).unwrap(),
//! };
//!
//! assert_eq!(example.bit_vector.len(), 8);
//! assert_eq!(example.bit_list.len(), 4);
//! assert_eq!(&example.variable_list[..], &[0, 1]);
//! assert_eq!(&example.fixed_vector[..], &[2, 3, 4, 5, 6, 7, 8, 9]);
//!
//! ```

#[macro_use]
mod fixed_vector;
mod fixed_vector_u8;
pub mod serde_utils;
mod tree_hash;
mod variable_list;
mod variable_list_u8;

pub use fixed_vector::FixedVector;
pub use fixed_vector_u8::FixedVectorU8;
pub use ssz::{BitList, BitVector, Bitfield};
pub use typenum;
pub use variable_list::VariableList;
pub use variable_list_u8::VariableListU8;

pub mod length {
    pub use ssz::{Fixed, Variable};
}

/// Returned when an item encounters an error.
#[derive(PartialEq, Debug, Clone)]
pub enum Error {
    OutOfBounds {
        i: usize,
        len: usize,
    },
    /// A `BitList` does not have a set bit, therefore it's length is unknowable.
    MissingLengthInformation,
    /// A `BitList` has excess bits set to true.
    ExcessBits,
    /// A `BitList` has an invalid number of bytes for a given bit length.
    InvalidByteCount {
        given: usize,
        expected: usize,
    },
}
