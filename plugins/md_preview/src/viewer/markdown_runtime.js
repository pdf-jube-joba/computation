import {from_text} from "./markdown_viewer.js";

let macrosPromise;

export function normalizePath(value) {
  return (value || "").trim().replace(/^\/+/, "");
}

export function requestHeaders() {
  return {
    "user-identity": "from_browser",
  };
}

export function currentFileUrl(path) {
  if (!path) {
    throw new Error("path is empty");
  }
  return `/${path}`;
}

export async function fetchTextFile(path) {
  const response = await fetch(currentFileUrl(path), {
    method: "GET",
    headers: requestHeaders(),
  });
  if (!response.ok) {
    throw new Error(`GET failed: ${response.status} ${response.statusText}`);
  }
  return response.text();
}

export async function loadMacros() {
  if (!macrosPromise) {
    macrosPromise = fetch(new URL("./macros.txt", window.location.href), {
      method: "GET",
      headers: requestHeaders(),
    })
      .then(response => {
        if (!response.ok) {
          throw new Error(`GET failed: ${response.status} ${response.statusText}`);
        }
        return response.text();
      })
      .then(from_text)
      .catch(error => {
        console.error("failed to load macros.txt", error);
        return {};
      });
  }

  return macrosPromise;
}
