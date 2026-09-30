# SSZ specification tests

Runs the JSON fixtures from [ssz-specs](https://github.com/ethereum/ssz-specs) releases.
Requires Rust/Cargo, Make, `curl`, `tar`, and `sha256sum` or `shasum`.

```sh
make test
```

The first run downloads the release pinned in the [Makefile](Makefile) to `.cache/`.
Each fixture file is one test. To run some of them, filter by name:

```sh
make test FILTER=compatible_unions
```

To try another release, pass its tag and the SHA-256 of its tarball:

```sh
make test RELEASE=v0.2.0 SHA256=<hash from ssz-test-vectors-v0.2.0.tar.gz.sha256>
```

To run your own fixtures, set `SSZ_SPEC_TESTS` to an absolute path:

```sh
SSZ_SPEC_TESTS=/path/to/fixtures cargo test -p ssz_spec_tests
```

Without `SSZ_SPEC_TESTS`, the fixture test is ignored and only the harness self-test runs.

To support a new fixture type, add its Rust type to `tests/spec/types.rs` if needed
and map its `typeName` in `tests/spec/dispatch.rs`.

`make clean` deletes downloaded fixtures.
