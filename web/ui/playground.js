import {loadBundleManifest, mountModel} from "./script.js";

const bundleBaseUrl = "/wasm_bundle/";
const assetBaseUrl = "/assets/";
const kind = document.body.dataset.kind;
const selectionParam = kind === "compiler" ? "compiler" : "model";

const modelSelect = document.querySelector("#model-select");
const reloadButton = document.querySelector("#reload-button");
const statusText = document.querySelector("#status-text");
const selectedModelText = document.querySelector("#selected-model");
const modelCountText = document.querySelector("#model-count");
const root = document.querySelector("#playground-root");
const runtimeLink = document.querySelector("#runtime-link");
const loaderLink = document.querySelector("#loader-link");
const wasmLink = document.querySelector("#wasm-link");

let availableModels = [];
let bundleManifest = {entries: [], loaders: {}};
let mountSequence = 0;

function setStatus(message, isError = false) {
  statusText.textContent = message;
  statusText.classList.toggle("is-error", isError);
}

function currentParams() {
  return new URL(window.location.href).searchParams;
}

function selectedModelFromLocation() {
  return (currentParams().get(selectionParam) || "").trim();
}

function updateLocation(model) {
  const url = new URL(window.location.href);
  if (model) {
    url.searchParams.set(selectionParam, model);
  } else {
    url.searchParams.delete(selectionParam);
  }
  window.history.replaceState(window.history.state, "", url);
}

function artifactUrl(fileName) {
  return new URL(fileName, window.location.origin + bundleBaseUrl).href;
}

function setArtifactLinks(model) {
  const entry = bundleManifest.entries.find(entry => entry.name === model);
  const entries = [
    [runtimeLink, bundleManifest.runtime],
    [loaderLink, bundleManifest.loaders[entry?.backend || "wasm"]],
    [wasmLink, "browser.bc.wasm.assets/"],
  ];
  for (const [link, fileName] of entries) {
    link.href = artifactUrl(fileName);
  }
}

async function fetchAvailableModels() {
  bundleManifest = await loadBundleManifest(bundleBaseUrl);
  const names = bundleManifest.entries.filter(entry => entry.kind === kind).map(entry => entry.name);

  return Array.from(new Set(names)).sort((left, right) => left.localeCompare(right));
}

function fillSelect(models, selectedModel) {
  modelSelect.replaceChildren();
  for (const model of models) {
    const option = document.createElement("option");
    option.value = model;
    option.textContent = model;
    option.selected = model === selectedModel;
    modelSelect.appendChild(option);
  }
}

async function loadSelectedModel(model) {
  const seq = ++mountSequence;
  root.classList.add("is-loading");
  root.replaceChildren();
  selectedModelText.textContent = model;
  setArtifactLinks(model);

  try {
    await mountModel(root, {
      model,
      bundleBaseUrl,
      assetBaseUrl,
    });
    if (seq !== mountSequence) {
      return;
    }
    setStatus(`Loaded ${model}`);
  } catch (error) {
    if (seq !== mountSequence) {
      return;
    }
    root.textContent = `failed to load ${model}: ${error}`;
    setStatus(`failed to load ${model}: ${error}`, true);
  } finally {
    if (seq === mountSequence) {
      root.classList.remove("is-loading");
    }
  }
}

async function selectModel(model) {
  const normalized = availableModels.includes(model) ? model : availableModels[0];
  if (!normalized) {
    setStatus(`no generated ${kind}s were found under ${bundleBaseUrl}`, true);
    return;
  }

  modelSelect.value = normalized;
  updateLocation(normalized);
  await loadSelectedModel(normalized);
}

async function bootstrap() {
  setStatus(`Loading ${kind} list...`);
  try {
    availableModels = await fetchAvailableModels();
    modelCountText.textContent = String(availableModels.length);
    if (availableModels.length === 0) {
      fillSelect([], "");
      selectedModelText.textContent = "-";
      setStatus(`no generated ${kind}s were found under ${bundleBaseUrl}`, true);
      return;
    }

    const requested = selectedModelFromLocation();
    const initial = availableModels.includes(requested) ? requested : availableModels[0];
    fillSelect(availableModels, initial);
    if (requested && requested !== initial) {
      setStatus(`${kind} ${requested} was not found; loaded ${initial} instead`, true);
    }
    await selectModel(initial);
  } catch (error) {
    setStatus(`failed to load ${kind} list: ${error}`, true);
  }
}

modelSelect.addEventListener("change", () => {
  void selectModel(modelSelect.value);
});

reloadButton.addEventListener("click", () => {
  void loadSelectedModel(modelSelect.value);
});

void bootstrap();
