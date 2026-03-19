import katex from "katex";
import remarkRehype from "remark-rehype";
import rehypeStringify from "rehype-stringify";
import {createMarkdownParser} from "./markdown_extensions.js";
import {createRemarkRehypeOptions} from "./markdown_to_hast.js";

const WASM_MOUNT_URL = __WASM_MOUNT_URL__;

export async function renderMarkdownToElement({text, element, basePath = "", macros = {}}) {
  const html = await renderMarkdownToHtml({text, basePath, macros});
  element.innerHTML = html;
  element.classList.add("md-view");
  await mountEmbeddedModels(element);
  return element;
}

export async function renderMarkdownToHtml({text, basePath = "", macros = {}}) {
  const katexMacros = normalizeKatexMacros(macros);
  const processor = createMarkdownParser()
    .use(remarkRehype, createRemarkRehypeOptions({
      basePath,
      katexMacros,
      renderKatexNode,
      renderMathError,
    }))
    .use(rehypeStringify, {allowDangerousHtml: true});

  const file = await processor.process(String(text || ""));
  return String(file);
}

export function from_text(text) {
  return parseKatexMacros(text);
}

function parseKatexMacros(text) {
  const macros = {};
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#") || line.startsWith("%")) {
      continue;
    }

    const separator = line.indexOf(":");
    if (separator === -1) {
      continue;
    }

    const key = line.slice(0, separator).trim();
    const value = line.slice(separator + 1).trim();
    if (!key || !value) {
      continue;
    }
    macros[key] = value;
  }

  return macros;
}

function normalizeKatexMacros(macros) {
  if (!macros || typeof macros !== "object" || Array.isArray(macros)) {
    return {};
  }
  return macros;
}

async function mountEmbeddedModels(root) {
  const {mountModel} = await import(wasmMountRuntimeUrl());
  const modelRoots = root.matches?.("[data-model]")
    ? [root]
    : Array.from(root.querySelectorAll("[data-model]"));

  await Promise.all(modelRoots.map(async modelRoot => {
    const model = (modelRoot.dataset.model || "").trim();
    if (!model) {
      modelRoot.textContent = "missing data-model";
      return;
    }

    try {
      await mountModel(modelRoot, {
        model,
        code: extractPlainScript(modelRoot, "default-code"),
        ainput: extractPlainScript(modelRoot, "default-ainput"),
        rinput: extractPlainScript(modelRoot, "default-rinput"),
        bundleBaseUrl: WASM_MOUNT_URL,
        assetBaseUrl: WASM_MOUNT_URL,
      });
    } catch (error) {
      modelRoot.textContent = `failed to load ${model}: ${error}`;
    }
  }));
}

function wasmMountRuntimeUrl() {
  return `${WASM_MOUNT_URL}script.js`;
}

function extractPlainScript(root, className) {
  const el = root.querySelector(`script.${className}[type="text/plain"]`);
  if (!el) {
    return "";
  }
  return el.textContent || "";
}

function renderKatexNode(node, macros, displayMode) {
  try {
    return katex.renderToString(node.value, {
      displayMode,
      throwOnError: true,
      macros,
      output: "html",
    });
  } catch (error) {
    return renderMathError(node.value, String(error), displayMode);
  }
}

function renderMathError(source, message, displayMode) {
  const tagName = displayMode ? "div" : "span";
  return `<${tagName} class="md-math-error"><span class="md-math-error-label">KaTeX Error</span><code>${escapeHtml(source)}</code><span class="md-math-error-message">${escapeHtml(message)}</span></${tagName}>`;
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}
