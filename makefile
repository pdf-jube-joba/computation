PYTHON ?= python3
MDBOOK ?= mdbook

.PHONY: wasm-build generate generate-clean md-build build serve

generate:
	$(PYTHON) "$(CURDIR)/generate.py" --mode incremental

generate-clean:
	$(PYTHON) "$(CURDIR)/generate.py" --mode clean

wasm-build:
	$(PYTHON) "$(CURDIR)/build.py"

md-build: generate-clean
	$(MDBOOK) build "$(CURDIR)/dist"

build: wasm-build md-build

serve: generate-clean
	$(MDBOOK) serve "$(CURDIR)/dist"
