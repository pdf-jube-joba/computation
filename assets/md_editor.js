const pathInput = document.querySelector("#path-input");
const loadButton = document.querySelector("#load-button");
const saveButton = document.querySelector("#save-button");
const preview = document.querySelector("#preview");
const editor = document.querySelector("#editor");
const statusText = document.querySelector("#status-text");

function normalizePath(value) {
  return value.trim().replace(/^\/+/, "");
}

function setStatus(message, isError = false) {
  statusText.textContent = message;
  statusText.classList.toggle("status-error", isError);
}

function updatePreview() {
  preview.textContent = editor.value;
}

function setBusy(busy) {
  loadButton.disabled = busy;
  saveButton.disabled = busy;
  pathInput.disabled = busy;
  editor.disabled = busy;
}

function currentFileUrl() {
  const path = normalizePath(pathInput.value);
  if (!path) {
    throw new Error("path is empty");
  }
  return `/${path}`;
}

function requestHeaders() {
  return {
    "user-identity": "from_browser",
  };
}

function updateUrlQuery(path) {
  const url = new URL(window.location.href);
  if (path) {
    url.searchParams.set("path", path);
  } else {
    url.searchParams.delete("path");
  }
  window.history.replaceState({}, "", url);
}

async function loadFile() {
  const path = normalizePath(pathInput.value);
  if (!path) {
    setStatus("Path is empty.", true);
    return;
  }

  setBusy(true);
  setStatus(`Loading ${path} ...`);
  try {
    const response = await fetch(currentFileUrl(), {
      method: "GET",
      headers: requestHeaders(),
    });
    if (!response.ok) {
      throw new Error(`GET failed: ${response.status} ${response.statusText}`);
    }

    const text = await response.text();
    editor.value = text;
    updatePreview();
    updateUrlQuery(path);
    setStatus(`Loaded ${path}.`);
  } catch (error) {
    setStatus(String(error), true);
  } finally {
    setBusy(false);
  }
}

async function saveFile() {
  const path = normalizePath(pathInput.value);
  if (!path) {
    setStatus("Path is empty.", true);
    return;
  }

  setBusy(true);
  setStatus(`Saving ${path} ...`);
  try {
    const response = await fetch(currentFileUrl(), {
      method: "PUT",
      headers: {
        ...requestHeaders(),
        "Content-Type": "text/plain; charset=utf-8",
      },
      body: editor.value,
    });

    if (!response.ok) {
      const detail = await response.text().catch(() => "");
      throw new Error(`PUT failed: ${response.status} ${detail || response.statusText}`);
    }

    updatePreview();
    updateUrlQuery(path);
    setStatus(`Saved ${path}.`);
  } catch (error) {
    setStatus(String(error), true);
  } finally {
    setBusy(false);
  }
}

loadButton.addEventListener("click", loadFile);
saveButton.addEventListener("click", saveFile);
editor.addEventListener("input", updatePreview);
pathInput.addEventListener("keydown", event => {
  if (event.key === "Enter") {
    event.preventDefault();
    void loadFile();
  }
});

const initialPath = normalizePath(new URL(window.location.href).searchParams.get("path") || "");
if (initialPath) {
  pathInput.value = initialPath;
  void loadFile();
} else {
  updatePreview();
  setStatus("Set a path and click Load.");
}
