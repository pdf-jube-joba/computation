PYTHON ?= python3
MDBOOK ?= mdbook

.PHONY: generate build serve

generate:
	$(PYTHON) "$(CURDIR)/generate.py"

build: generate
	$(MDBOOK) build "$(CURDIR)/dist"

serve: generate
	$(MDBOOK) serve "$(CURDIR)/dist"
