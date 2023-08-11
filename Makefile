
.PHONY: install-dev
install-dev:
	cd anytree-cli && cargo install --profile dev --path .

.PHONY: install
install:
	cd anytree-cli && cargo install --profile release --path .

.PHONY: fmt
fmt:
	taplo fmt
	cargo +nightly fmt --all -v

.PHONY: fix
fix:
	cargo clippy --fix --allow-dirty

.PHONY: run
run:
	cargo run --bin anytree -- build "./hack/cargo_sbom.cdx.json"

.PHONY: run-mac
run-mac:
	cargo run --bin anytree -- build "./hack/cargo_sbom_mac.cdx.json"

.PHONY: debug_run
debug_run:
	ANYTREE_LOG=trace cargo run --bin anytree -- build "./hack/cargo_sbom.cdx.json"
