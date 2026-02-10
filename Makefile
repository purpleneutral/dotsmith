PREFIX ?= $(HOME)/.local
BINDIR  = $(PREFIX)/bin
MANDIR  = $(PREFIX)/share/man/man1
BINARY  = dotsmith

.PHONY: build install uninstall test clean check help man

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
	@if target/release/$(BINARY) mangen > /dev/null 2>&1; then \
		mkdir -p $(MANDIR); \
		target/release/$(BINARY) mangen > $(MANDIR)/dotsmith.1; \
		echo "  installed man page to $(MANDIR)/dotsmith.1"; \
	fi
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
	rm -f $(MANDIR)/dotsmith.1
	@echo "  removed $(BINDIR)/$(BINARY)"

man: build ## Generate man page
	target/release/$(BINARY) mangen > dotsmith.1
	@echo "  generated dotsmith.1"

test: ## Run all tests
	cargo test

check: ## Run clippy and tests
	cargo clippy -- -D warnings
	cargo test

clean: ## Remove build artifacts
	cargo clean
