.PHONY: lint
lint: ## Run linter
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: test
test: ## Run all tests
	cargo nextest run
	cargo test --doc

.PHONY: doc
doc: ## Build and open docs
	cargo doc --open

.PHONY: docker
docker: ## Build docker image
	docker build -t "zero2prod" .
