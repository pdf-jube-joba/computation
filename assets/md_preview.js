const pathText = document.querySelector("#path-text");
const preview = document.querySelector("#preview");
const statusText = document.querySelector("#status-text");

function normalizePath(value) {
  return (value || "").trim().replace(/^\/+/, "");
}

function setStatus(message, isError = false) {
  statusText.textContent = message;
  statusText.classList.toggle("status-error", isError);
}

function currentFileUrl(path) {
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

async function loadFile(path) {
  if (!path) {
    setStatus("Query parameter `path` is required.", true);
    return;
  }

  pathText.textContent = path;
  setStatus(`Loading ${path} ...`);
  try {
    const response = await fetch(currentFileUrl(path), {
      method: "GET",
      headers: requestHeaders(),
    });
    if (!response.ok) {
      throw new Error(`GET failed: ${response.status} ${response.statusText}`);
    }

    const text = await response.text();
    preview.textContent = text;
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
  preview.textContent = "";
  setStatus("Query parameter `path` is required.", true);
}
