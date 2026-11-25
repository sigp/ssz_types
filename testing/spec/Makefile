# The release used locally and in CI, and its tarball's SHA-256. Change both together.
PINNED_RELEASE := v0.1.0
PINNED_SHA256  := e2a65f032b59835c26127295293ea1bc07d7ca0ea1fe0e4f1128dffed333f878

RELEASE ?= $(PINNED_RELEASE)
# The pinned hash only applies to the pinned release.
SHA256  ?= $(if $(filter $(PINNED_RELEASE),$(RELEASE)),$(PINNED_SHA256))

TARBALL   := ssz-test-vectors-$(RELEASE).tar.gz
ifeq ($(SHA256),)
$(error RELEASE=$(RELEASE) also needs SHA256=<hash from $(TARBALL).sha256>)
endif
URL       := https://github.com/ethereum/ssz-specs/releases/download/$(RELEASE)/$(TARBALL)
FIXTURES  := .cache/$(RELEASE)
SHA256SUM := $(shell command -v sha256sum 2>/dev/null || echo shasum -a 256)

.PHONY: test clean

test: $(FIXTURES)
	SSZ_SPEC_TESTS="$(abspath $(FIXTURES))/fixtures" cargo test -p ssz_spec_tests -- $(FILTER)

# Extract into a temporary directory so an interrupted download is never used.
$(FIXTURES):
	rm -rf $@.tmp && mkdir -p $@.tmp
	curl -fsSL --retry 3 --proto '=https' -o $@.tmp/$(TARBALL) $(URL)
	echo "$(SHA256)  $@.tmp/$(TARBALL)" | $(SHA256SUM) -c
	tar -xzf $@.tmp/$(TARBALL) -C $@.tmp
	rm $@.tmp/$(TARBALL)
	mv $@.tmp $@

clean:
	rm -rf .cache
