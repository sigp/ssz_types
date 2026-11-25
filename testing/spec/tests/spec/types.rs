//! Rust types that match ssz-specs/tests/fillers/ssz.

use super::fixture::decode_hex;
use alloy_primitives::{U128, U256};
use serde_json::Value;
use ssz::{Bitfield, BitfieldBehaviour, ProgressiveBitList};
use ssz_derive::{Decode, Encode};
use ssz_types::{
    typenum::*, BitList, BitVector, FixedVector, ProgressiveVariableList, VariableList,
};
use tree_hash_derive::TreeHash;

/// Build the expected value from JSON without decoding the SSZ bytes.
pub trait FromFixture: Sized {
    fn from_fixture(value: &Value) -> Result<Self, String>;
}

macro_rules! integers {
    ($($ty:ty),+ $(,)?) => {$(
        impl FromFixture for $ty {
            fn from_fixture(value: &Value) -> Result<Self, String> {
                let text = match value {
                    Value::String(text) => text.clone(),
                    Value::Number(number) => number.to_string(),
                    _ => return Err(format!("expected an integer, got {value}")),
                };
                text.parse().map_err(|e| format!("invalid {}: {e}", stringify!($ty)))
            }
        }
    )+};
}

integers!(u8, u16, u32, u64, U128, U256);

impl FromFixture for bool {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        value
            .as_bool()
            .ok_or_else(|| format!("expected a boolean, got {value}"))
    }
}

impl<T: FromFixture> FromFixture for Vec<T> {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        let data = value.get("data").unwrap_or(value);
        match data {
            Value::Array(values) => values.iter().map(T::from_fixture).collect(),
            Value::String(text) => decode_hex(text)?
                .into_iter()
                .map(|byte| T::from_fixture(&Value::from(byte)))
                .collect(),
            _ => Err(format!("expected collection data, got {data}")),
        }
    }
}

impl<T: FromFixture, N: Unsigned> FromFixture for FixedVector<T, N> {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        Self::new(Vec::from_fixture(value)?).map_err(|e| format!("{e:?}"))
    }
}

impl<T: FromFixture, N: Unsigned> FromFixture for VariableList<T, N> {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        Self::new(Vec::from_fixture(value)?).map_err(|e| format!("{e:?}"))
    }
}

impl<T: FromFixture, N: Unsigned> FromFixture for ProgressiveVariableList<T, N> {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        Self::new(Vec::from_fixture(value)?).map_err(|e| format!("{e:?}"))
    }
}

fn set_bits<B: BitfieldBehaviour>(
    mut field: Bitfield<B>,
    bits: Vec<bool>,
) -> Result<Bitfield<B>, String> {
    if field.len() != bits.len() {
        return Err(format!("expected {} bits, got {}", field.len(), bits.len()));
    }
    for (index, bit) in bits.into_iter().enumerate() {
        field.set(index, bit).map_err(|e| format!("{e:?}"))?;
    }
    Ok(field)
}

impl<N: Unsigned> FromFixture for BitVector<N> {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        set_bits(Self::new(), Vec::from_fixture(value)?)
    }
}

impl<N: Unsigned> FromFixture for BitList<N> {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        let bits = Vec::from_fixture(value)?;
        let field = Self::with_capacity(bits.len()).map_err(|e| format!("{e:?}"))?;
        set_bits(field, bits)
    }
}

impl FromFixture for ProgressiveBitList {
    fn from_fixture(value: &Value) -> Result<Self, String> {
        let bits = Vec::from_fixture(value)?;
        set_bits(Self::with_capacity(bits.len()), bits)
    }
}

// These macros define types and read JSON values.
// The library derives handle encoding, decoding, and tree roots.
macro_rules! container {
    ($(#[$attr:meta])* $name:ident { $($field:ident: $ty:ty),+ $(,)? }) => {
        #[derive(Debug, PartialEq, Encode, Decode, TreeHash)]
        $(#[$attr])*
        pub struct $name { $($field: $ty),+ }

        impl FromFixture for $name {
            fn from_fixture(value: &Value) -> Result<Self, String> {
                Ok(Self { $($field: <$ty>::from_fixture(
                    value.get(stringify!($field))
                        .ok_or_else(|| format!("missing field {}", stringify!($field)))?
                )?),+ })
            }
        }
    };
}

macro_rules! compatible_union {
    ($name:ident { $($variant:ident($ty:ty) = $selector:literal),+ $(,)? }) => {
        #[derive(Debug, PartialEq, Encode, Decode, TreeHash)]
        #[ssz(enum_behaviour = "compatible_union")]
        #[tree_hash(enum_behaviour = "compatible_union")]
        pub enum $name {
            $(
                #[ssz(selector = $selector)]
                #[tree_hash(selector = $selector)]
                $variant($ty),
            )+
        }

        impl FromFixture for $name {
            fn from_fixture(value: &Value) -> Result<Self, String> {
                let selector = u8::from_fixture(&value["selector"])?;
                match selector.to_string().as_str() {
                    $($selector =>
                        Ok(Self::$variant(<$ty>::from_fixture(&value["data"])?)),)+
                    _ => Err(format!("unknown {} selector {selector}", stringify!($name))),
                }
            }
        }
    };
}

pub type Bytes32 = FixedVector<u8, U32>;
pub type SampleUint16List4 = VariableList<u16, U4>;
pub type SampleUint16ProgressiveList = ProgressiveVariableList<u16>;
pub type SampleUint64ProgressiveList = ProgressiveVariableList<u64>;
pub type SampleBytes32ProgressiveList = ProgressiveVariableList<Bytes32>;
pub type SampleNestedProgressiveList = ProgressiveVariableList<ProgressiveVariableList<u16>>;
pub type SampleSquareProgressiveList = ProgressiveVariableList<SampleSquare>;
pub type SampleShapeProgressiveList = ProgressiveVariableList<SampleShape>;

container!(SampleContainerWithProgressiveList {
    a: u16,
    b: SampleUint64ProgressiveList,
    c: u8,
});

container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(1, 0, 1))]
    SampleSquare {
        side: u16,
        color: u8
    }
);
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(0, 1, 1))]
    SampleCircle {
        radius: u16,
        color: u8
    }
);
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(1))]
    SampleOneField { a: u16 }
);
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(0, 0, 1))]
    SampleLeadingGaps { c: u32 }
);
container!(
    #[tree_hash(
        struct_behaviour = "progressive_container",
        active_fields(1, 0, 0, 1, 0, 1)
    )]
    SampleMultipleGaps {
        a: u8,
        b: u16,
        c: u32
    }
);
container!(
    #[tree_hash(
        struct_behaviour = "progressive_container",
        active_fields(
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        )
    )]
    SampleWidestLayout { tail: u8 }
);
container!(
    #[tree_hash(
        struct_behaviour = "progressive_container",
        active_fields(1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1)
    )]
    SampleLevelBoundary {
        first: u16,
        last: u8
    }
);
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(1, 0, 1))]
    SampleBoundedListField {
        head: u64,
        body: SampleUint16List4
    }
);
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(1, 1, 1))]
    SampleProgressiveFields {
        head: u64,
        numbers: SampleUint64ProgressiveList,
        flags: ProgressiveBitList,
    }
);
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(1, 0, 1))]
    SampleInnerShape { x: u16, y: u8 }
);
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(1, 0, 1))]
    SampleOuterShape {
        head: u8,
        inner: SampleInnerShape
    }
);

// Two suites use "SampleShapeContainer" for different types.
container!(ProgressiveShapeContainer {
    tag: u8,
    shape: SampleSquare
});
container!(UnionShapeContainer {
    tag: u64,
    body: SampleShape
});

compatible_union!(SampleShape {
    Square(SampleSquare) = "1",
    Circle(SampleCircle) = "2",
    SquareAlias(SampleSquare) = "127",
});
compatible_union!(SampleNumbers {
    List(SampleUint16List4) = "1",
    Alias(SampleUint16List4) = "2",
});
compatible_union!(SampleEmptyProne {
    Squares(SampleSquareProgressiveList) = "1",
    Circles(ProgressiveVariableList<SampleCircle>) = "2",
});
compatible_union!(SampleSquareOnly { Square(SampleSquare) = "5" });
compatible_union!(SampleNestedShape {
    Shape(SampleShape) = "1",
    SquareOnly(SampleSquareOnly) = "2",
});
container!(
    #[tree_hash(struct_behaviour = "progressive_container", active_fields(1, 0, 1))]
    SampleShapeProgressiveContainer {
        tag: u64,
        body: SampleShape
    }
);
