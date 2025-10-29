MAKEFLAGS += --no-print-directory

e2e-chain-up:
	@echo "--> Starting e2e test blockchain environment"
	@docker compose -f e2e/docker-compose.yml up -d --wait

e2e-chain-down:
	@echo "--> Stopping e2e test blockchain environment"
	@docker compose -f e2e/docker-compose.yml down -v

.PHONY: e2e-chain-up e2e-chain-down

#### Build ####

build:
	@echo "--> Building contracts"
	@cargo wasm

build-opt:
	@echo "--> Checking for protoc installation"
	@$(MAKE) check-protoc
	@echo "--> Checking for wasm-opt installation"
	@$(MAKE) check-wasm-opt
	@echo "--> Checking for cosmwasm-check installation"
	@$(MAKE) check-cosmwasm-check
	@echo "--> Building optimized contracts"
	@RUSTFLAGS="-C link-arg=-s" cargo wasm --locked
	@echo "--> Optimizing wasm binaries"
	@mkdir -p ./artifacts/
	@for f in target/wasm32-unknown-unknown/release/*.wasm; do wasm-opt -Os "$$f" -o "artifacts/$$(basename "$$f")"; done
	@echo "--> Optimized wasm binaries are in the ./artifacts/ directory"
	@echo "--> Running cosmwasm-check on optimized binaries"
	@for f in artifacts/*.wasm; do cosmwasm-check "$$f"; done

clean:
	@echo "--> Cleaning build artifacts"
	@cargo clean

check-protoc:
	@command -v protoc >/dev/null 2>&1 || { echo "protoc not found; install with: sudo apt install protobuf-compiler (or your distro's package)"; exit 1; }

check-wasm-opt:
	@command -v wasm-opt >/dev/null 2>&1 || { echo "wasm-opt not found; install with: sudo apt install binaryen (or your distro's package)"; exit 1; }

check-cosmwasm-check:
	@command -v cosmwasm-check >/dev/null 2>&1 || { echo "cosmwasm-check not found; install with: cargo install cosmwasm-check"; exit 1; }

.PHONY: build clean build-opt check-protoc check-wasm-opt check-cosmwasm-check