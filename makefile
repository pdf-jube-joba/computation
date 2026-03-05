PYTHON ?= python3
MDBOOK ?= mdbook
WATCHEXEC ?= watchexec

.PHONY: wasm-build generate md-build build md-serve md-serve-once serve

generate:
	$(PYTHON) "$(CURDIR)/generate.py"

wasm-build:
	$(PYTHON) "$(CURDIR)/build.py"

md-build: generate
	$(MDBOOK) build "$(CURDIR)/dist"

build: wasm-build md-build

md-serve-once: generate
	$(MDBOOK) serve "$(CURDIR)/dist"

md-serve:
	$(WATCHEXEC) --restart \
		--watch "$(CURDIR)/docs" \
		--watch "$(CURDIR)/models" \
		-- "$(MAKE)" --no-print-directory md-serve-once

serve: md-serve
