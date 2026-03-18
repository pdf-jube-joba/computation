import {renderMarkdownToElement} from "/md-preview-assets/generated/markdown_viewer.js";
import {fetchTextFile, loadMacros, normalizePath} from "./markdown_runtime.js";

const pathText = document.querySelector("#path-text");
const preview = document.querySelector("#preview");
const statusText = document.querySelector("#status-text");

function setStatus(message, isError = false) {
  statusText.textContent = message;
  statusText.classList.toggle("status-error", isError);
}

async function loadFile(path) {
  if (!path) {
    setStatus("Query parameter `path` is required.", true);
    return;
  }

  pathText.textContent = path;
  setStatus(`Loading ${path} ...`);
  try {
    const [text, macros] = await Promise.all([fetchTextFile(path), loadMacros()]);
    await renderMarkdownToElement({
      text,
      element: preview,
      basePath: path,
      macros,
    });
    setStatus(`Loaded ${path}.`);
  } catch (error) {
    setStatus(String(error), true);
  }
}

const initialPath = normalizePath(new URL(window.location.href).searchParams.get("path") || "");
if (initialPath) {
  void loadFile(initialPath);
} else {
  pathText.textContent = "(missing)";
  preview.innerHTML = "";
  setStatus("Query parameter `path` is required.", true);
}
