# md_preview

This directory contains the browser-side markdown preview build setup.
Source code lives in `src/`, and `build.mjs` bundles it into an explicitly specified output directory.
The build script requires `--out-dir <path>` and writes all generated files under that directory.
It also accepts `--wasm-mount-url <url-prefix>` for the mounted wasm assets used by embedded models.
It generates `link_index.json` alongside the browser assets by scanning all repository markdown files.
The built output includes the viewer HTML/CSS/JS from `src/viewer/`.
When `WORKSPACE_FS_PLUGIN_SETTINGS_JSON` contains `[plugin.md_preview]`, `build.mjs` also generates `enhance_runner.js` from `[[plugin.md_preview.enhance]]`.
When used through `workspace_fs`, mount the plugin output and serve it from `/md-preview-assets/`.
The plugin also depends on the wasm bundle plugin and expects its mount URL to be passed in.
It renders markdown to HTML in the browser with `remark` / `rehype`.
It also adds local extensions for KaTeX-style math written as `\(...\)` / `\[...\]`.
GitHub-style alerts such as `> [!NOTE]` and `> [!TIP]` are converted to custom HTML blocks.
KaTeX macros are not loaded implicitly by the viewer.
Callers are expected to load `macros.txt`, convert it with `from_text(text)`, and pass the result as `macros`.
Run `npm install` once in this directory if dependencies are missing, then run `node ./build.mjs --out-dir <path> --wasm-mount-url /wasm_bundle/` after changing `src/`.
For `workspace_fs` plugins, use `deps = ["build-wasm"]` and pass `{OUTPUT_DIRECTORY}` plus `{MOUNT_BUILD_WASM}`.
