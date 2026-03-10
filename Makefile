# Delegate all rules to sub directories.

Makefile:;

.DEFAULT_GOAL := .DEFAULT

.PHONY: %

notice:
	$(MAKE) -C pg_documentdb_gw notice
# Rust gateway components
PGRX_VERSION := 0.16.0
GATEWAY_DIR := pg_documentdb_gw_host
GATEWAY_SOURCE := $(GATEWAY_DIR)
# Build profile: release or dev (default: release)
GATEWAY_PROFILE ?= release

check-no-distributed:
	$(MAKE) -C pg_documentdb_core check
	$(MAKE) -C pg_documentdb check

install-no-distributed: install-documentdb

install-documentdb:
	$(MAKE) -C pg_documentdb_core install
	$(MAKE) -C pg_documentdb install
	$(MAKE) -C pg_documentdb_extended_rum install

# Install Rust gateway
install-gateway:
	@echo "Installing Rust gateway (profile: $(GATEWAY_PROFILE))..."
	@cd $(GATEWAY_SOURCE) && \
	if ! command -v cargo >/dev/null 2>&1; then \
		echo "Error: cargo not found. Please install Rust first."; \
		exit 1; \
	fi; \
	if ! cargo pgrx --version 2>/dev/null | grep -q "$(PGRX_VERSION)"; then \
		echo "Installing cargo-pgrx $(PGRX_VERSION)..."; \
		cargo install cargo-pgrx --version $(PGRX_VERSION) --locked; \
	fi; \
	PG_VERSION=$$(pg_config --version | awk '{print $$2}' | cut -d. -f1); \
	echo "Building gateway for PostgreSQL $$PG_VERSION..."; \
	if [ "$(GATEWAY_PROFILE)" = "dev" ]; then \
		cargo pgrx install --sudo --pg-config $$(which pg_config); \
	else \
		cargo pgrx install --sudo --pg-config $$(which pg_config) --release; \
	fi

# Install gateway in debug mode
install-gateway-debug:
	$(MAKE) install-gateway GATEWAY_PROFILE=dev

# Clean Rust gateway
clean-gateway:
	@echo "Cleaning Rust gateway..."
	@cd $(GATEWAY_SOURCE) && cargo clean

# Install everything including gateway
install-all: install-documentdb install-gateway

.DEFAULT:
	$(MAKE) -C pg_documentdb_core
	$(MAKE) -C pg_documentdb
	$(MAKE) -C pg_documentdb_extended_rum
	$(MAKE) -C internal/pg_documentdb_distributed

%:
	$(MAKE) -C pg_documentdb_core $@
	$(MAKE) -C pg_documentdb $@
	$(MAKE) -C pg_documentdb_extended_rum $@
	$(MAKE) -C internal/pg_documentdb_distributed $@
