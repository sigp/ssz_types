//! Compare SSZ encodings, decodings, and tree roots with a fixture.

use super::{
    fixture::{decode_hex, Fixture, Info},
    types::FromFixture,
};
use ssz::{Decode, Encode};
use std::fmt::Debug;
use tree_hash::TreeHash;

fn byte_window(bytes: &[u8], offset: usize) -> String {
    let start = offset.saturating_sub(8).min(bytes.len());
    let end = offset.saturating_add(9).min(bytes.len());
    format!(
        "{} bytes, window {start}..{end}: 0x{}",
        bytes.len(),
        hex::encode(&bytes[start..end])
    )
}

fn equal_bytes(label: &str, actual: &[u8], expected: &[u8]) -> Result<(), String> {
    if actual == expected {
        return Ok(());
    }
    let offset = actual
        .iter()
        .zip(expected)
        .position(|(a, b)| a != b)
        .unwrap_or(actual.len().min(expected.len()));
    Err(format!(
        "{label}: first difference at byte {offset}\n  expected {}\n  got      {}",
        byte_window(expected, offset),
        byte_window(actual, offset),
    ))
}

pub fn check<T: Decode + Encode + TreeHash + FromFixture + PartialEq + Debug>(
    fixture: &Fixture,
) -> Result<(), String> {
    if let Some(reason) = &fixture.rejection_reason {
        let raw = fixture
            .raw_bytes
            .as_deref()
            .ok_or("a rejection fixture needs rawBytes")?;
        // Python and Rust name decode errors differently, so any error passes.
        if T::from_ssz_bytes(&decode_hex(raw)?).is_ok() {
            return Err(format!("decoder accepted input marked {reason}"));
        }
        return Ok(());
    }

    let serialized = decode_hex(&fixture.serialized)?;
    let root = decode_hex(&fixture.root)?;
    let expected = T::from_fixture(&fixture.value)?;
    equal_bytes("encoding", &expected.as_ssz_bytes(), &serialized)?;
    if expected.ssz_bytes_len() != serialized.len() {
        return Err(format!(
            "ssz_bytes_len: expected {}, got {}",
            serialized.len(),
            expected.ssz_bytes_len()
        ));
    }
    equal_bytes("root", expected.tree_hash_root().as_slice(), &root)?;

    let decoded = T::from_ssz_bytes(&serialized)
        .map_err(|e| format!("decoder rejected valid bytes: {e:?}"))?;
    if decoded != expected {
        return Err(format!(
            "decoded value: expected {expected:?}, got {decoded:?}"
        ));
    }
    Ok(())
}

/// Released fixtures only show that `check` passes. Show that it can also fail.
pub fn self_test() -> Result<(), String> {
    fn boolean(serialized: &str, root: &str) -> Fixture {
        Fixture {
            type_name: "Boolean".into(),
            value: true.into(),
            serialized: serialized.into(),
            root: root.into(),
            raw_bytes: None,
            rejection_reason: None,
            info: Info {
                fixture_format: "ssz_test".into(),
            },
        }
    }
    fn rejection(bytes: &str) -> Fixture {
        Fixture {
            raw_bytes: Some(bytes.into()),
            rejection_reason: Some("INVALID".into()),
            ..boolean(bytes, "")
        }
    }

    let true_root = format!("0x01{}", "00".repeat(31));
    let zero_root = format!("0x{}", "00".repeat(32));
    let cases = [
        ("valid", boolean("0x01", &true_root), true),
        ("wrong encoding", boolean("0x00", &true_root), false),
        ("wrong root", boolean("0x01", &zero_root), false),
        ("invalid input", rejection("0x02"), true),
        ("valid input marked invalid", rejection("0x01"), false),
    ];
    for (name, fixture, should_pass) in cases {
        let result = check::<bool>(&fixture);
        if result.is_ok() != should_pass {
            return Err(format!("{name}: check returned {result:?}"));
        }
    }
    Ok(())
}
