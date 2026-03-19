import {renderMarkdownToElement} from "/md-preview-assets/markdown_viewer.js";
import {
  currentFileUrl,
  fetchTextFile,
  loadMacros,
  normalizePath,
  requestHeaders,
} from "./markdown_runtime.js";

const pathText = document.querySelector("#path-text");
const preview = document.querySelector("#preview");
const navigation = document.querySelector("#navigation");
const sidebarToggle = document.querySelector("#sidebar-toggle");
const sidebarClose = document.querySelector("#sidebar-close");
const statusText = document.querySelector("#status-text");
const app = document.querySelector(".app");
const sidebarStateKey = "md-preview-sidebar-collapsed";

function setStatus(message, isError = false) {
  statusText.textContent = message;
  statusText.classList.toggle("status-error", isError);
}

function setSidebarCollapsed(collapsed) {
  app.classList.toggle("sidebar-collapsed", collapsed);
  sidebarToggle.textContent = collapsed ? "Show Navigation" : "Hide Navigation";
  sidebarToggle.setAttribute("aria-expanded", String(!collapsed));
  window.localStorage.setItem(sidebarStateKey, collapsed ? "1" : "0");
}

function initializeSidebarState() {
  const collapsed = window.localStorage.getItem(sidebarStateKey) === "1";
  setSidebarCollapsed(collapsed);
}

function splitPath(path) {
  const normalized = normalizePath(path);
  const lastSlash = normalized.lastIndexOf("/");
  if (lastSlash === -1) {
    return {
      directory: "",
      name: normalized,
    };
  }

  return {
    directory: normalized.slice(0, lastSlash),
    name: normalized.slice(lastSlash + 1),
  };
}

function joinPath(directory, entry) {
  const normalizedDirectory = normalizePath(directory);
  const normalizedEntry = normalizePath(entry);
  if (!normalizedDirectory) {
    return normalizedEntry;
  }
  if (!normalizedEntry) {
    return normalizedDirectory;
  }
  return `${normalizedDirectory}/${normalizedEntry}`;
}

function directoryUrl(directory) {
  const normalizedDirectory = normalizePath(directory);
  return normalizedDirectory ? `/${normalizedDirectory}/` : "/";
}

function previewHref(path) {
  return `./md_preview.html?path=${encodeURIComponent(path)}`;
}

function isExternalLink(path) {
  return /^[a-z][a-z0-9+.-]*:/i.test(path);
}

function toDisplayPath(path) {
  return normalizePath(path) || "/";
}

async function fetchOptionalText(path) {
  const response = await fetch(currentFileUrl(normalizePath(path)), {
    method: "GET",
    headers: requestHeaders(),
  });
  if (response.status === 404) {
    return null;
  }
  if (!response.ok) {
    throw new Error(`GET failed: ${response.status} ${response.statusText}`);
  }
  return response.text();
}

async function fetchDirectoryEntries(directory) {
  const response = await fetch(directoryUrl(directory), {
    method: "GET",
    headers: requestHeaders(),
  });
  if (!response.ok) {
    throw new Error(`GET failed: ${response.status} ${response.statusText}`);
  }

  const text = await response.text();
  if (!text.trim()) {
    return [];
  }
  return text.split("\n").map(entry => entry.trim()).filter(Boolean);
}

function parseNavigationEntries(json, directory) {
  const rawItems = Array.isArray(json)
    ? json
    : Array.isArray(json?.items)
      ? json.items
      : null;
  if (!rawItems) {
    throw new Error("navigation.json must be an array or an object with an `items` array.");
  }

  return rawItems.map((item, index) => {
    if (typeof item === "string") {
      return {
        label: item,
        path: joinPath(directory, item),
      };
    }

    if (!item || typeof item !== "object") {
      throw new Error(`navigation.json entry ${index + 1} is invalid.`);
    }

    const label = String(item.name ?? item.title ?? item.label ?? item.path ?? item.link ?? item.href ?? "");
    const rawPath = item.path ?? item.link ?? item.href;
    if (!label || typeof rawPath !== "string" || !rawPath.trim()) {
      throw new Error(`navigation.json entry ${index + 1} must have a name and link target.`);
    }

    return {
      label,
      path: isExternalLink(rawPath) || rawPath.startsWith("/")
        ? rawPath
        : joinPath(directory, rawPath),
    };
  });
}

async function loadNavigation(currentPath) {
  const {directory} = splitPath(currentPath);
  const navigationPath = joinPath(directory, "navigation.json");
  try {
    const navigationText = await fetchOptionalText(navigationPath);
    if (navigationText !== null) {
      const parsed = JSON.parse(navigationText);
      return parseNavigationEntries(parsed, directory);
    }
  } catch (error) {
    console.warn(`failed to load ${navigationPath}, falling back to directory listing`, error);
  }

  const entries = await fetchDirectoryEntries(directory);
  return entries.map(entry => ({
    label: entry,
    path: joinPath(directory, entry),
  }));
}

function createNavigationLink(item, currentPath) {
  const listItem = document.createElement("li");
  listItem.className = "navigation-item";

  const label = document.createElement("span");
  label.className = "navigation-label";
  label.textContent = item.label;

  const normalizedCurrent = normalizePath(currentPath);
  const normalizedItemPath = normalizePath(item.path);
  const isCurrent = normalizedItemPath === normalizedCurrent;

  const meta = document.createElement("span");
  meta.className = "navigation-meta";
  meta.textContent = toDisplayPath(item.path);

  if (isCurrent) {
    const current = document.createElement("span");
    current.className = "navigation-current";
    current.append(label, meta);
    listItem.append(current);
    return listItem;
  }

  const link = document.createElement("a");
  link.className = "navigation-link";

  if (isExternalLink(item.path)) {
    link.href = item.path;
    link.target = "_blank";
    link.rel = "noreferrer";
  } else if (item.path.endsWith(".md")) {
    const targetPath = normalizePath(item.path);
    link.href = previewHref(targetPath);
    link.dataset.previewPath = targetPath;
  } else if (item.path.endsWith("/")) {
    link.href = directoryUrl(item.path);
  } else if (item.path.startsWith("/")) {
    link.href = item.path;
  } else {
    link.href = `/${normalizePath(item.path)}`;
  }

  link.append(label, meta);
  listItem.append(link);
  return listItem;
}

function renderNavigation(items, currentPath) {
  navigation.innerHTML = "";

  if (!items.length) {
    const empty = document.createElement("div");
    empty.className = "navigation-empty";
    empty.textContent = "No entries found.";
    navigation.append(empty);
    return;
  }

  const list = document.createElement("ul");
  list.className = "navigation-list";
  for (const item of items) {
    list.append(createNavigationLink(item, currentPath));
  }
  navigation.append(list);
}

async function loadFile(path) {
  if (!path) {
    setStatus("Query parameter `path` is required.", true);
    return;
  }

  const normalizedPath = normalizePath(path);
  pathText.textContent = normalizedPath;
  setStatus(`Loading ${normalizedPath} ...`);
  try {
    const [text, macros, navigationItems] = await Promise.all([
      fetchTextFile(normalizedPath),
      loadMacros(),
      loadNavigation(normalizedPath),
    ]);
    await renderMarkdownToElement({
      text,
      element: preview,
      basePath: normalizedPath,
      macros,
    });
    renderNavigation(navigationItems, normalizedPath);
    setStatus(`Loaded ${normalizedPath}.`);
  } catch (error) {
    preview.innerHTML = "";
    navigation.innerHTML = "";
    setStatus(String(error), true);
  }
}

function currentPathFromLocation() {
  return normalizePath(new URL(window.location.href).searchParams.get("path") || "");
}

async function navigateTo(path, {pushHistory = true} = {}) {
  const normalizedPath = normalizePath(path);
  if (!normalizedPath) {
    pathText.textContent = "(missing)";
    preview.innerHTML = "";
    navigation.innerHTML = "";
    setStatus("Query parameter `path` is required.", true);
    return;
  }

  if (pushHistory) {
    const url = new URL(window.location.href);
    url.searchParams.set("path", normalizedPath);
    window.history.pushState({path: normalizedPath}, "", url);
  }

  await loadFile(normalizedPath);
}

navigation.addEventListener("click", event => {
  const link = event.target.closest("a[data-preview-path]");
  if (!link) {
    return;
  }

  event.preventDefault();
  void navigateTo(link.dataset.previewPath);
});

sidebarToggle.addEventListener("click", () => {
  setSidebarCollapsed(!app.classList.contains("sidebar-collapsed"));
});

sidebarClose.addEventListener("click", () => {
  setSidebarCollapsed(true);
});

window.addEventListener("popstate", () => {
  void navigateTo(currentPathFromLocation(), {pushHistory: false});
});

initializeSidebarState();
void navigateTo(currentPathFromLocation(), {pushHistory: false});
