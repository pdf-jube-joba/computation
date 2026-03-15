PYTHON ?= python3
MDBOOK ?= mdbook
WATCHEXEC ?= watchexec

.PHONY: wasm-build generate generate-clean md-build build watch-generate serve

generate:
	$(PYTHON) "$(CURDIR)/generate.py" --mode incremental

generate-clean:
	$(PYTHON) "$(CURDIR)/generate.py" --mode clean

wasm-build:
	$(PYTHON) "$(CURDIR)/build.py"

md-build: generate-clean
	$(MDBOOK) build "$(CURDIR)/dist"

build: wasm-build md-build

watch-generate:
	@set -eu; \
	watch_args=''; \
	for path in "$(CURDIR)/docs" "$(CURDIR)/models" "$(CURDIR)/wasm_bundle" "$(CURDIR)/README.md" "$(CURDIR)/TODO.md"; do \
		if [ -e "$$path" ]; then \
			watch_args="$$watch_args --watch $$path"; \
		fi; \
	done; \
	exec $(WATCHEXEC) $$watch_args -- "$(PYTHON)" "$(CURDIR)/generate.py" --mode incremental

serve: generate-clean
	@set -eu; \
	watch_args=''; \
	for path in "$(CURDIR)/docs" "$(CURDIR)/models" "$(CURDIR)/wasm_bundle" "$(CURDIR)/README.md" "$(CURDIR)/TODO.md"; do \
		if [ -e "$$path" ]; then \
			watch_args="$$watch_args --watch $$path"; \
		fi; \
	done; \
	watch_pid=''; \
	trap 'if [ -n "$$watch_pid" ]; then kill "$$watch_pid"; wait "$$watch_pid" 2>/dev/null || true; fi' EXIT INT TERM; \
	$(WATCHEXEC) $$watch_args -- "$(PYTHON)" "$(CURDIR)/generate.py" --mode incremental & \
	watch_pid=$$!; \
	$(MDBOOK) serve "$(CURDIR)/dist"
