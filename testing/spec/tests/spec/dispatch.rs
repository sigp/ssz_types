//! Map fixture names to Rust SSZ types.

use super::{checks::check, fixture::Fixture, types::*};
use alloy_primitives::{U128 as Uint128, U256 as Uint256};
use ssz::ProgressiveBitList;
use ssz_types::{typenum::*, BitList, BitVector, FixedVector, VariableList};

/// Fixture IDs name the filler module: `tests/fillers/ssz/test_<suite>.py::<case>`.
fn suite_from_id(id: &str) -> Result<&str, String> {
    id.split_once("::")
        .and_then(|(module, _)| module.rsplit('/').next())
        .and_then(|module| module.strip_prefix("test_"))
        .and_then(|module| module.strip_suffix(".py"))
        .ok_or_else(|| format!("invalid fixture ID: {id}"))
}

pub fn run_case(id: &str, fixture: &Fixture) -> Result<(), String> {
    // A new format may reuse these fields with different meanings, so reject it.
    let format = &fixture.info.fixture_format;
    if format != "ssz_test" {
        return Err(format!("unsupported fixture format: {format}"));
    }

    macro_rules! dispatch {
        ($($name:literal => $ty:ty),+ $(,)?) => {
            match fixture.type_name.as_str() {
                $($name => check::<$ty>(fixture),)+
                // In v0.1.0, two suites define different types with this name, so match the
                // suite too. ethereum/ssz-specs#149 renames the progressive one to
                // `SampleSquareContainer`. After bumping past v0.1.0, map both names directly
                // and delete this arm and `suite_from_id`.
                "SampleShapeContainer" => match suite_from_id(id)? {
                    "progressive_containers" => check::<ProgressiveShapeContainer>(fixture),
                    "compatible_unions" => check::<UnionShapeContainer>(fixture),
                    suite => Err(format!("unknown SampleShapeContainer in {suite}")),
                },
                name => Err(format!("unknown fixture type: {name}")),
            }
        };
    }

    dispatch! {
        "Boolean" => bool,
        "Uint8" => u8,
        "Uint16" => u16,
        "Uint32" => u32,
        "Uint64" => u64,
        "Uint128" => Uint128,
        "Uint256" => Uint256,
        "Bytes4" => FixedVector<u8, U4>,
        "Bytes32" => Bytes32,
        "Bytes52" => FixedVector<u8, U52>,
        "Bytes64" => FixedVector<u8, U64>,
        "ByteList512KiB" => VariableList<u8, U524288>,
        "SampleBitVector8" => BitVector<U8>,
        "SampleBitVector64" => BitVector<U64>,
        "SampleBitList16" => BitList<U16>,
        "SampleUint16Vector3" => FixedVector<u16, U3>,
        "SampleUint64Vector4" => FixedVector<u64, U4>,
        "SampleUint32List16" => VariableList<u32, U16>,
        "SampleBytes32List8" => VariableList<Bytes32, U8>,
        "BoundaryBitVector1" => BitVector<U1>,
        "BoundaryBitVector7" => BitVector<U7>,
        "BoundaryBitVector9" => BitVector<U9>,
        "BoundaryBitVector255" => BitVector<U255>,
        "BoundaryBitVector256" => BitVector<U256>,
        "BoundaryBitVector257" => BitVector<U257>,
        "BoundaryBitList256" => BitList<U256>,
        "BoundaryUint64List32" => VariableList<u64, U32>,
        "SmokeBitList8" => BitList<U8>,
        "SampleUint64ProgressiveList" => SampleUint64ProgressiveList,
        "SampleBytes32ProgressiveList" => SampleBytes32ProgressiveList,
        "SampleUint16ProgressiveList" => SampleUint16ProgressiveList,
        "SampleNestedProgressiveList" => SampleNestedProgressiveList,
        "ProgressiveBitList" => ProgressiveBitList,
        "SampleContainerWithProgressiveList" => SampleContainerWithProgressiveList,
        "SampleSquare" => SampleSquare,
        "SampleCircle" => SampleCircle,
        "SampleOneField" => SampleOneField,
        "SampleLeadingGaps" => SampleLeadingGaps,
        "SampleMultipleGaps" => SampleMultipleGaps,
        "SampleWidestLayout" => SampleWidestLayout,
        "SampleLevelBoundary" => SampleLevelBoundary,
        "SampleBoundedListField" => SampleBoundedListField,
        "SampleProgressiveFields" => SampleProgressiveFields,
        "SampleOuterShape" => SampleOuterShape,
        "SampleSquareProgressiveList" => SampleSquareProgressiveList,
        "SampleShape" => SampleShape,
        "SampleNumbers" => SampleNumbers,
        "SampleEmptyProne" => SampleEmptyProne,
        "SampleNestedShape" => SampleNestedShape,
        "SampleShapeProgressiveContainer" => SampleShapeProgressiveContainer,
        "SampleShapeProgressiveList" => SampleShapeProgressiveList,
    }
}
