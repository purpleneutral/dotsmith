PREFIX ?= $(HOME)/.local
BINDIR  = $(PREFIX)/bin
BINARY  = dotsmith

.PHONY: build install uninstall test clean check help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build release binary
	@command -v cargo >/dev/null 2>&1 || { \
		echo "error: cargo not found. Install Rust: https://rustup.rs"; exit 1; }
	cargo build --release

install: build ## Install to ~/.local/bin (override with PREFIX=/usr/local)
	@mkdir -p $(BINDIR)
	cp target/release/$(BINARY) $(BINDIR)/$(BINARY)
	chmod 755 $(BINDIR)/$(BINARY)
	@echo ""
	@echo "  installed $(BINARY) to $(BINDIR)/$(BINARY)"
	@echo ""
	@case ":$$PATH:" in \
		*":$(BINDIR):"*) ;; \
		*) echo "  warning: $(BINDIR) is not in your PATH"; \
		   echo "  add this to your shell profile:"; \
		   echo "    export PATH=\"$(BINDIR):\$$PATH\""; \
		   echo "" ;; \
	esac

uninstall: ## Remove dotsmith from install location
	rm -f $(BINDIR)/$(BINARY)
	@echo "  removed $(BINDIR)/$(BINARY)"

test: ## Run all tests
	cargo test

check: ## Run clippy and tests
	cargo clippy -- -D warnings
	cargo test

clean: ## Remove build artifacts
	cargo clean
