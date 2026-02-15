PYTHON ?= python3
MDBOOK ?= mdbook

.PHONY: summary build serve
summary:
	$(PYTHON) "$(CURDIR)/generate.py"

build: summary
	$(MDBOOK) build "$(CURDIR)"

serve: summary
	$(MDBOOK) serve "$(CURDIR)"
