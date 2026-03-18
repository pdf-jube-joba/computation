# md_preview

This directory contains the browser-side markdown preview build setup.
Source code lives in `src/`, and `npm run build` bundles it into `../assets/generated/markdown_viewer.js`.
The build script requires `--out-dir <path>` and writes all generated files under that directory.
The generated viewer is used by `assets/md_preview.js` and `assets/md_editor.js`.
When used through `workspace_fs`, mount the plugin output and serve it from `/md-preview-assets/`.
It renders markdown to HTML in the browser with `remark` / `rehype`.
It also adds local extensions for KaTeX-style math written as `\(...\)` / `\[...\]`.
GitHub-style alerts such as `> [!NOTE]` and `> [!TIP]` are converted to custom HTML blocks.
KaTeX macros are not loaded implicitly by the viewer.
Callers are expected to load `assets/macros.txt`, convert it with `from_text(text)`, and pass the result as `macros`.
Run `npm install` once in this directory if dependencies are missing, then run `npm run build` after changing `src/`.
For `workspace_fs` plugins, pass `{OUTPUT_DIRECTORY}` as the `--out-dir` value.
