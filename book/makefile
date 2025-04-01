BOOK_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
ASSETS := $(BOOK_ROOT)/src/assets/generated

MODEL_WEB_DIRS := $(wildcard $(BOOK_ROOT)../models/*/web)
.DEFAULT_GOAL := full_serve

.PHONY: build serve copy_assets build_models

non_full_build: copy_assets
	mdbook build $(BOOK_DIR)

non_full_serve: copy_assets
	mdbook serve $(BOOK_DIR)

full_build: build_models copy_assets
	mdbook build $(BOOK_DIR)

full_serve: build_models copy_assets
	mdbook serve $(BOOK_DIR)

build_models:
	@echo "📦 Building models $(MODEL_WEB_DIRS)"
	@for dir in $(MODEL_WEB_DIRS); do \
		if [ -f $$dir/Cargo.toml ]; then \
			echo "📦 Building $$dir..."; \
			wasm-pack build $$dir --target web --out-dir $$dir/pkg; \
		fi \
	done

copy_assets:
	@echo "📦 Copying assets from $(MODEL_WEB_DIRS) to $(ASSETS)"
	@mkdir -p $(ASSETS)
	@for dir in $(MODEL_WEB_DIRS); do \
		MODEL=$$(basename $$(dirname $$dir)); \
		if [ -d $$dir/pkg ]; then \
			echo "📦 Copying $$dir/pkg..."; \
			cp $$dir/pkg/*.js $(ASSETS); \
			cp $$dir/pkg/*.wasm $(ASSETS); \
		fi; \
		if [ -f $$dir/$${MODEL}_glue.js ]; then \
			echo "📦 Copying $$dir/$${MODEL}_glue.js..."; \
			cp $$dir/$${MODEL}_glue.js $(ASSETS); \
		fi; \
	done
