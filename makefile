PYTHON ?= python3
MDBOOK ?= mdbook
WATCHEXEC ?= watchexec

.PHONY: generate build serve serve-once

generate:
	$(PYTHON) "$(CURDIR)/generate.py"

build: generate
	$(MDBOOK) build "$(CURDIR)/dist"

serve-once: generate
	$(MDBOOK) serve "$(CURDIR)/dist"

serve:
	$(WATCHEXEC) --restart \
		--watch "$(CURDIR)/docs" \
		--watch "$(CURDIR)/models" \
		-- "$(MAKE)" --no-print-directory serve-once
