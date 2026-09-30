//! Run the JSON fixtures from ssz-specs, one test per fixture file.
//!
//! `make test` downloads the pinned release and sets SSZ_SPEC_TESTS to its
//! fixture directory. Without it, the fixture test is ignored.

mod checks;
mod dispatch;
mod fixture;
mod types;

use fixture::Fixture;
use libtest_mimic::{Arguments, Trial};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn main() {
    let args = Arguments::from_args();
    let mut trials = vec![Trial::test("harness::check_can_fail", || {
        Ok(checks::self_test()?)
    })];

    match std::env::var_os("SSZ_SPEC_TESTS") {
        Some(root) => {
            let root = PathBuf::from(root);
            let mut files = Vec::new();
            fixture_files(&root, &mut files).unwrap_or_else(|e| panic!("{e}"));
            assert!(!files.is_empty(), "no JSON fixtures in {}", root.display());
            files.sort();
            for path in files {
                let name = trial_name(&root, &path);
                trials.push(Trial::test(name, move || Ok(run_file(&path)?)));
            }
        }
        None => trials.push(
            Trial::test("fixtures", || {
                Err("set SSZ_SPEC_TESTS or run `make -C testing/spec test`".into())
            })
            .with_ignored_flag(true),
        ),
    }

    libtest_mimic::run(&args, trials).exit();
}

fn fixture_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.is_dir() {
            fixture_files(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
    Ok(())
}

/// `ssz/ssz/test_basic_types/test_boolean_true.json` becomes
/// `ssz::ssz::test_basic_types::test_boolean_true`.
fn trial_name(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path).with_extension("");
    let parts: Vec<_> = relative.iter().map(|part| part.to_string_lossy()).collect();
    parts.join("::")
}

fn run_file(path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let fixtures: BTreeMap<String, Fixture> =
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    if fixtures.is_empty() {
        return Err(format!("{} contains no fixtures", path.display()));
    }
    for (id, fixture) in &fixtures {
        dispatch::run_case(id, fixture).map_err(|e| format!("{id}\n{e}"))?;
    }
    Ok(())
}
