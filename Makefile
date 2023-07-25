
.PHONY: install-dev
install-dev:
	cd anytree-cli && cargo install --profile dev --path .

.PHONY: install
install:
	cd anytree-cli && cargo install --profile release --path .

.PHONY: fmt
fmt:
	cargo +nightly fmt --all -v
