# === 各種パス設定 ===
BOOK_DIR := book
MODEL_WEB_DIRS := $(wildcard models/*/web)
ASSETS_OUT := $(BOOK_DIR)/src/assets/generated

# === デフォルトターゲット ===
.DEFAULT_GOAL := build

.PHONY: serve build build_book build_models copy_assets

# === ファイル監視して自動リビルドは .sh にまかせる ===
serve:
	bash ./serve.sh

# === ビルド ===
build: build_models copy_assets build_book

build_book:
	mdbook build $(BOOK_DIR)

# === wasm-pack によるビルドを行う ===
build_models:
	@for dir in $(MODEL_WEB_DIRS); do \
		if [ -f $$dir/Cargo.toml ]; then \
			echo "📦 wasm-pack build $$dir"; \
			wasm-pack build $$dir --target web --out-dir $$dir/pkg; \
		fi; \
	done

# === wasm-pack ビルド出力物等（.js を手書きしたものも含めて） assets/ へのコピー ===
copy_assets:
	@mkdir -p $(ASSETS_OUT)
	@for dir in $(MODEL_WEB_DIRS); do \
		MODEL=$$(basename $$(dirname $$dir)); \
		OUT_DIR=$(ASSETS_OUT)/$$MODEL; \
		echo "📁 Copying assets for $$MODEL to $$OUT_DIR"; \
		mkdir -p $$OUT_DIR; \
		mkdir -p $$OUT_DIR/pkg; \
		if [ -d $$dir/pkg ]; then \
			cp $$dir/pkg/*.js $$OUT_DIR/pkg 2>/dev/null || true; \
			cp $$dir/pkg/*.wasm $$OUT_DIR/pkg 2>/dev/null || true; \
		fi; \
		if [ -f $$dir/$${MODEL}_glue.js ]; then \
			cp $$dir/$${MODEL}_glue.js $$OUT_DIR; \
		fi; \
	done
