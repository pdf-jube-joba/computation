const OUTPUT_PLACEHOLDER = "\u00a0";
const bundleManifests = new Map();

export function loadBundleManifest(baseUrl) {
  const url = new URL('./manifest.json', resolveBaseUrl(baseUrl)).href;
  if (!bundleManifests.has(url)) {
    const promise = fetch(url).then(response => {
      if (!response.ok) {
        throw new Error(`GET ${url} failed: ${response.status} ${response.statusText}`);
      }
      return response.json();
    });
    bundleManifests.set(url, promise);
    promise.catch(() => bundleManifests.delete(url));
  }
  return bundleManifests.get(url);
}

export function ensureStyleSheet(options = {}) {
  const already = document.querySelector('link[data-wm-style="true"]');
  if (already) {
    return already;
  }

  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = assetUrl("./style.css", options.assetBaseUrl);
  link.dataset.wmStyle = "true";
  document.head.appendChild(link);
  return link;
}

export async function mountModel(element, options = {}) {
  ensureStyleSheet(options);
  const spec = normalizeMountSpec(options);
  const manifest = await loadBundleManifest(options.bundleBaseUrl);
  const entry = manifest.entries.find(entry => entry.name === spec.model);
  if (!entry) throw new Error(`Unknown model/compiler: ${spec.model}`);
  const runtime = await import(assetUrl(manifest.runtime, options.bundleBaseUrl));
  if (entry.kind === "model") {
    const instance = await runtime.createModel(entry.name);
    await renderModelMount(element, spec.model, instance, spec, options);
    return;
  }
  if (entry.kind === "compiler") {
    const instance = await runtime.createCompiler(entry.name);
    renderCompilerMount(element, spec.model, instance, spec);
    return;
  }
  throw new Error(`unknown model/compiler kind for ${spec.model}: ${entry.kind}`);
}

function normalizeMountSpec(options) {
  const model = (options.model || "").trim();
  if (!model) {
    throw new Error("model is required");
  }
  if (model.includes("=>")) {
    throw new Error(`pipeline syntax is not supported: ${model}`);
  }
  return {
    model,
    code: options.code || "",
    ainput: options.ainput || "",
    rinput: options.rinput || "",
  };
}

function assetUrl(relativePath, baseUrl) {
  return new URL(relativePath, resolveBaseUrl(baseUrl)).href;
}

function resolveBaseUrl(baseUrl) {
  if (!baseUrl) {
    return import.meta.url;
  }
  return new URL(baseUrl, window.location.href);
}

function setOutputText(el, value) {
  const text = value ?? "";
  el.textContent = text === "" ? OUTPUT_PLACEHOLDER : text;
}

function parseJsonString(value, name) {
  if (typeof value !== "string") {
    throw new Error(`${name} returned non-string value`);
  }
  try {
    return JSON.parse(value);
  } catch (error) {
    throw new Error(`${name} returned invalid JSON: ${error}`);
  }
}

function createPanel(host, className, title) {
  const panel = document.createElement("section");
  panel.className = `wm-panel ${className}`.trim();
  const heading = document.createElement("div");
  heading.className = "wm-panel-heading";
  heading.textContent = title;
  const body = document.createElement("div");
  body.className = "wm-panel-body";
  panel.append(heading, body);
  host.appendChild(panel);
  return {panel, body};
}

function createField(host, label, className, defaultValue = "", rows = 6) {
  const field = document.createElement("div");
  field.className = `wm-field ${className}`.trim();
  const area = document.createElement("textarea");
  area.rows = rows;
  area.value = defaultValue;
  if (label) {
    const labelEl = document.createElement("label");
    labelEl.textContent = label;
    field.append(labelEl);
  }
  field.append(area);
  host.appendChild(field);
  return area;
}

function createOutput(host, title, className = "") {
  const wrap = document.createElement("div");
  wrap.className = `wm-output-box ${className}`.trim();
  const label = document.createElement("div");
  label.className = "wm-output-label";
  label.textContent = title;
  const body = document.createElement("div");
  body.className = "wm-output";
  wrap.append(label, body);
  host.appendChild(wrap);
  return body;
}

async function drawCurrentSnapshot(instance, renderer) {
  const snapshot = await instance.snapshot();
  const rendered = await instance.render(snapshot);
  renderer.draw(parseJsonString(rendered, "render"));
}

async function renderModelMount(root, name, instance, defaults, options) {
  root.replaceChildren();
  const statusPanel = createPanel(root, "wm-panel-controller-status", `Model: ${name}`);
  const status = createOutput(statusPanel.body, "Status", "wm-controller-status-box");
  setOutputText(status, "");

  const boundary = createPanel(root, "wm-panel-boundary", "Boundary");
  const boundaryGrid = document.createElement("div");
  boundaryGrid.className = "wm-model-boundary-grid";
  boundary.body.appendChild(boundaryGrid);
  const left = document.createElement("div");
  left.className = "wm-model-boundary-left";
  const right = document.createElement("div");
  right.className = "wm-model-boundary-right";
  boundaryGrid.append(left, right);

  const code = createField(left, "Code", "wm-field-code", defaults.code, 12);
  const ainput = createField(right, "AInput", "wm-field-ainput", defaults.ainput, 5);
  const foutput = createOutput(right, "FOutput", "wm-output-final-box");
  setOutputText(foutput, "");
  const createBtn = document.createElement("button");
  createBtn.className = "wm-model-create-button";
  createBtn.textContent = "Create";
  right.appendChild(createBtn);

  const transition = createPanel(root, "wm-panel-transition", "Transition");
  const transitionGrid = document.createElement("div");
  transitionGrid.className = "wm-model-transition-grid";
  transition.body.appendChild(transitionGrid);
  const rinput = createField(transitionGrid, "RInput", "wm-field-rinput", defaults.rinput, 5);
  const routput = createOutput(transitionGrid, "ROutput", "wm-output-runtime-box");
  setOutputText(routput, "");
  const stepBtn = document.createElement("button");
  stepBtn.className = "wm-model-step-button";
  stepBtn.textContent = "Step";
  transitionGrid.appendChild(stepBtn);

  const snapshotPanel = createPanel(root, "wm-panel-state", "Snapshot");
  const snapshotContainer = document.createElement("div");
  snapshotContainer.className = "wm-state wm-snapshot-view";
  snapshotPanel.body.appendChild(snapshotContainer);

  const {Renderer} = await import(assetUrl("./renderer.js", options.assetBaseUrl));
  const renderer = new Renderer(snapshotContainer);
  let machineReady = false;

  const setStatus = (message, isError = false) => {
    status.style.color = isError ? "red" : "";
    setOutputText(status, message || "");
  };

  createBtn.addEventListener("click", async () => {
    try {
      await instance.make(code.value, ainput.value);
      await drawCurrentSnapshot(instance, renderer);
      machineReady = true;
      setOutputText(routput, "");
      setOutputText(foutput, "");
      setStatus("");
    } catch (error) {
      machineReady = false;
      setStatus(`create failed: ${error}`, true);
    }
  });

  stepBtn.addEventListener("click", async () => {
    if (!machineReady) {
      setStatus("machine not initialized; run Create first", true);
      return;
    }
    try {
      const result = parseJsonString(await instance.step(rinput.value), "step");
      if (result.kind === "continue") {
        await drawCurrentSnapshot(instance, renderer);
        setOutputText(routput, result.routput ?? "");
        setOutputText(foutput, "");
        setStatus("");
        return;
      }
      if (result.kind === "halt") {
        machineReady = false;
        setOutputText(foutput, result.foutput ?? "");
        setStatus("halted");
        return;
      }
      throw new Error(`unknown step result kind: ${result.kind}`);
    } catch (error) {
      setStatus(`step failed: ${error}`, true);
    }
  });
}

function renderCompilerMount(root, name, instance, defaults) {
  root.replaceChildren();
  const statusPanel = createPanel(root, "wm-panel-controller-status", `Compiler: ${name}`);
  const status = createOutput(statusPanel.body, "Status", "wm-controller-status-box");
  setOutputText(status, "");

  const panel = createPanel(root, "wm-panel-boundary", "Compiler IO");
  const addIoRow = (
    label,
    actionLabel,
    className,
    defaultValue,
    action,
    rows = 4,
    direction = "source-to-target",
  ) => {
    const row = document.createElement("div");
    row.className = "wm-compiler-row";

    const rowTitle = document.createElement("div");
    rowTitle.className = "wm-output-label";
    rowTitle.textContent = label;
    rowTitle.style.gridColumn = "1 / -1";
    row.appendChild(rowTitle);

    const sourceWrap = document.createElement("div");
    sourceWrap.className = "wm-compiler-side wm-compiler-source";
    const sourceTitle = document.createElement("div");
    sourceTitle.className = "wm-output-label";
    sourceTitle.textContent = "(Source)";
    sourceWrap.appendChild(sourceTitle);

    const middle = document.createElement("div");
    middle.className = "wm-compiler-middle";
    const button = document.createElement("button");
    button.className = "wm-compiler-action";
    button.textContent = actionLabel;
    middle.appendChild(button);

    const targetWrap = document.createElement("div");
    targetWrap.className = "wm-compiler-side wm-compiler-target";
    const targetTitle = document.createElement("div");
    targetTitle.className = "wm-output-label";
    targetTitle.textContent = "(Target)";
    targetWrap.appendChild(targetTitle);

    let sourceInput = null;
    let targetInput = null;
    let sourceOutput = null;
    let targetOutput = null;

    if (direction === "source-to-target") {
      sourceInput = createField(sourceWrap, "", className, defaultValue, rows);
      const outPre = document.createElement("pre");
      outPre.className = "wm-output";
      const outCode = document.createElement("code");
      setOutputText(outCode, "");
      outPre.appendChild(outCode);
      targetWrap.appendChild(outPre);
      targetOutput = outCode;
    } else {
      const outPre = document.createElement("pre");
      outPre.className = "wm-output";
      const outCode = document.createElement("code");
      setOutputText(outCode, "");
      outPre.appendChild(outCode);
      sourceWrap.appendChild(outPre);
      sourceOutput = outCode;
      targetInput = createField(targetWrap, "", className, defaultValue, rows);
    }

    button.addEventListener("click", async () => {
      try {
        if (direction === "source-to-target") {
          setOutputText(targetOutput, await action(sourceInput.value));
        } else {
          setOutputText(sourceOutput, await action(targetInput.value));
        }
        setStatus("");
      } catch (error) {
        setStatus(`${actionLabel} failed: ${error}`, true);
      }
    });

    row.append(sourceWrap, middle, targetWrap);
    panel.body.appendChild(row);
  };

  const setStatus = (message, isError = false) => {
    status.style.color = isError ? "red" : "";
    setOutputText(status, message || "");
  };

  addIoRow("Code", "compile", "wm-field-code", defaults.code, async value => instance.compileCode(value), 8);
  addIoRow("AInput", "encode", "wm-field-ainput", defaults.ainput, async value => instance.encodeAinput(value), 6);
  addIoRow("RInput", "encode", "wm-field-rinput", defaults.rinput, async value => instance.encodeRinput(value), 4);
  addIoRow("ROutput", "decode", "wm-field-routput", "", async value => instance.decodeRoutput(value), 4, "target-to-source");
  addIoRow("FOutput", "decode", "wm-field-foutput", "", async value => instance.decodeFoutput(value), 4, "target-to-source");
}
