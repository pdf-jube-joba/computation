// assets/script.js
// モデルごとの wasm バンドルとレンダラーを読み込み、簡単な UI を構築する。

// -------------------------------------
// ヘルパー: style.css を一度だけ適用
// -------------------------------------
function ensureStyleSheet() {
  const already = document.querySelector('link[data-wm-style="true"]');
  if (already) return;
  const href = new URL("./style.css", import.meta.url).href;
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = href;
  link.dataset.wmStyle = "true";
  document.head.appendChild(link);
}
ensureStyleSheet();

const STEP_FAILED_RESULT = { output: undefined, stepped: false, halted: false };

const COMPILER_ENCODE_OPS = {
  code: compiler => value => compiler.compileCode(value),
  ainput: compiler => value => compiler.compileAInput(value),
  rinput: compiler => value => compiler.compileRInput(value),
};

const COMPILER_DECODE_OPS = {
  routput: compiler => value => compiler.decodeROutput(value),
  foutput: compiler => value => compiler.decodeFOutput(value),
};

function wmLog(...args) {
  console.log("[wm]", ...args);
}

function buildCompilerEdgeMetas(names) {
  const metas = [];
  for (let i = 0; i + 1 < names.length; i += 1) {
    const sourceName = names[i];
    const targetName = names[i + 1];
    metas.push({
      source: { name: sourceName, index: i },
      target: { name: targetName, index: i + 1 },
      key: `${sourceName}->${targetName}#${i}`,
      compilerName: `${sourceName}-${targetName}`,
    });
  }
  return metas;
}

// -------------------------------------
// ヘルパー: <script type="text/plain" class="..."> からデフォルト文字列を取得
// -------------------------------------
function extractPlainScript(root, className) {
  const el = root.querySelector(`script.${className}[type="text/plain"]`);
  if (!el) return "";
  return el.textContent || "";
}

// -------------------------------------
// ヘルパー: 子要素を確保 (なければ生成)
// -------------------------------------
function ensureChild(root, selector, tagName, className) {
  let el = root.querySelector(selector);
  if (!el) {
    el = document.createElement(tagName);
    if (className) el.classList.add(className);
    root.appendChild(el);
  }
  return el;
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
  return { panel, body };
}

function createLabeledTextarea(host, { label, classNames = [], defaultValue = "", rows = 6 }) {
  const field = document.createElement("div");
  field.className = "wm-field";
  for (const token of classNames) {
    if (token) field.classList.add(token);
  }
  const labelEl = document.createElement("label");
  labelEl.textContent = label;
  const area = document.createElement("textarea");
  area.rows = rows;
  area.value = defaultValue;
  field.append(labelEl, area);
  host.appendChild(field);
  return { field, area };
}

function createOutputBox(host, { title, classNames = [], bodyClassNames = [] }) {
  const wrap = document.createElement("div");
  wrap.className = "wm-output-box";
  for (const token of classNames) {
    if (token) wrap.classList.add(token);
  }
  const label = document.createElement("div");
  label.className = "wm-output-label";
  label.textContent = title;
  const body = document.createElement("div");
  body.className = "wm-output";
  for (const token of bodyClassNames) {
    if (token) body.classList.add(token);
  }
  wrap.append(label, body);
  host.appendChild(wrap);
  return { wrap, body };
}

// -------------------------------------
// MachineWrapper: wasm モジュールのロードとラップ
// -------------------------------------
class MachineWrapper {
  static instanceCounter = 0;

  constructor(modelName) {
    this.modelName = modelName;
    this.instanceId = MachineWrapper.instanceCounter++;
    this.module = null;
    this.createFn = null;
    this.stepFn = null;
    this.currentFn = null;
  }

  async init() {
    if (this.module) return;

    const wasmPath = `./wasm_bundle/${this.modelName}.js?instance=${this.instanceId}`;
    const module = await import(wasmPath);
    this.module = module;
    this.createFn = typeof module.create === "function" ? module.create : null;
    this.stepFn = typeof module.step_machine === "function" ? module.step_machine : null;
    this.currentFn = typeof module.current_machine === "function" ? module.current_machine : null;
    const missing = [];
    if (!this.createFn) missing.push("create");
    if (!this.stepFn) missing.push("step_machine");
    if (!this.currentFn) missing.push("current_machine");
    if (missing.length) {
      throw new Error(`WASM module ${this.modelName} is missing exports: ${missing.join(", ")}`);
    }
    if (typeof module.default === "function") {
      await module.default();
    }
  }

  assertReady() {
    if (!this.module) {
      throw new Error(`Machine "${this.modelName}" is not initialized`);
    }
  }

  async createMachine(codeStr, ainputStr) {
    this.assertReady();
    return Promise.resolve(this.createFn(codeStr, ainputStr));
  }

  async stepMachine(runtimeInputStr) {
    this.assertReady();
    return Promise.resolve(this.stepFn(runtimeInputStr));
  }

  async currentState() {
    this.assertReady();
    return Promise.resolve(this.currentFn());
  }
}

// -------------------------------------
// CompilerWrapper: compiler wasm モジュールのロードとラップ
// -------------------------------------
class CompilerWrapper {
  static instanceCounter = 0;

  constructor(compilerName) {
    this.compilerName = compilerName;
    this.instanceId = CompilerWrapper.instanceCounter++;
    this.module = null;
    this.compileCodeFn = null;
    this.compileAInputFn = null;
    this.compileRInputFn = null;
    this.decodeROutputFn = null;
    this.decodeFOutputFn = null;
  }

  async init() {
    if (this.module) return;
    const wasmPath = `./wasm_bundle/${this.compilerName}.js?instance=${this.instanceId}`;
    const module = await import(wasmPath);
    this.module = module;
    this.compileCodeFn = typeof module.compile_code === "function" ? module.compile_code : null;
    this.compileAInputFn = typeof module.compile_ainput === "function" ? module.compile_ainput : null;
    this.compileRInputFn = typeof module.compile_rinput === "function" ? module.compile_rinput : null;
    this.decodeROutputFn = typeof module.decode_routput === "function" ? module.decode_routput : null;
    this.decodeFOutputFn = typeof module.decode_foutput === "function" ? module.decode_foutput : null;
    const missing = [];
    if (!this.compileCodeFn) missing.push("compile_code");
    if (!this.compileAInputFn) missing.push("compile_ainput");
    if (!this.compileRInputFn) missing.push("compile_rinput");
    if (!this.decodeROutputFn) missing.push("decode_routput");
    if (!this.decodeFOutputFn) missing.push("decode_foutput");
    if (missing.length) {
      throw new Error(`Compiler WASM is missing exports: ${missing.join(", ")}`);
    }
    if (typeof module.default === "function") {
      await module.default();
    }
  }

  async compileCode(code) {
    return Promise.resolve(this.compileCodeFn(code));
  }

  async compileAInput(ainput) {
    return Promise.resolve(this.compileAInputFn(ainput));
  }

  async compileRInput(rinput) {
    return Promise.resolve(this.compileRInputFn(rinput));
  }

  async decodeROutput(output) {
    return Promise.resolve(this.decodeROutputFn(output));
  }

  async decodeFOutput(output) {
    return Promise.resolve(this.decodeFOutputFn(output));
  }
}

function parsePipelineSpec(text) {
  return (text || "")
    .split("=>")
    .map(s => s.trim())
    .filter(Boolean);
}

class MachineNodeView {
  constructor(host, {
    title,
    role,
    machineName = null,
    defaults = {},
    runtimeControls = false,
    runtimeLabels = false,
    reportStatus = null,
    reportError = null,
    transformROutput = null,
    transformFOutput = null,
  }) {
    this.role = role;
    this.machineName = machineName;
    this.reportStatus = reportStatus || (() => { });
    this.reportError = reportError || (() => { });
    this.transformROutput = transformROutput || (async value => value);
    this.transformFOutput = transformFOutput || (async value => value);
    this.wrapper = null;
    this.renderer = null;
    this.machineState = "uninitialized"; // uninitialized | ready | machine_setted | init_failed
    this.autoEnabled = false;
    this.autoTimerId = null;

    this.buildShell(title, role);
    this.buildInputFields(defaults, runtimeLabels);
    this.buildOutputAreas();

    if (runtimeControls) {
      this.buildRuntimeControls();
    }

    host.appendChild(this.element);
  }

  buildShell(title, role) {
    this.element = document.createElement("section");
    this.element.className = `wm-node wm-node-${role}`;
    this.element.dataset.nodeRole = role;

    this.titleEl = document.createElement("div");
    this.titleEl.className = "wm-node-title";
    this.titleEl.textContent = title;
    this.element.appendChild(this.titleEl);

    this.boundaryPanel = createPanel(this.element, "wm-panel-boundary", "Boundary");
    this.transitionPanel = createPanel(this.element, "wm-panel-transition", "Transition");
    this.statePanel = createPanel(this.element, "wm-panel-state", "Snapshot");

    this.boundaryMainRow = document.createElement("div");
    this.boundaryMainRow.className = "wm-boundary-row wm-boundary-row-main";
    this.boundaryActionsRow = document.createElement("div");
    this.boundaryActionsRow.className = "wm-boundary-row wm-boundary-row-actions";
    this.boundaryPanel.body.append(this.boundaryMainRow, this.boundaryActionsRow);

    this.transitionMainRow = document.createElement("div");
    this.transitionMainRow.className = "wm-transition-row wm-transition-row-main";
    this.transitionActionsRow = document.createElement("div");
    this.transitionActionsRow.className = "wm-transition-row wm-transition-row-actions";
    this.transitionPanel.body.append(
      this.transitionMainRow,
      this.transitionActionsRow,
    );
  }

  buildInputFields(defaults, runtimeLabels) {
    const codeField = createLabeledTextarea(this.boundaryMainRow, {
      label: runtimeLabels ? "Code (runtime)" : "Code",
      classNames: ["wm-field-code", "wm-boundary-code"],
      defaultValue: defaults.code || "",
      rows: 8,
    });
    const ainputField = createLabeledTextarea(this.boundaryMainRow, {
      label: runtimeLabels ? "AInput (runtime)" : "AInput",
      classNames: ["wm-field-ainput", "wm-boundary-ainput"],
      defaultValue: defaults.ainput || "",
      rows: 6,
    });
    const rinputField = createLabeledTextarea(this.transitionMainRow, {
      label: runtimeLabels ? "RInput (runtime)" : "RInput",
      classNames: ["wm-field-rinput", "wm-transition-rinput"],
      defaultValue: defaults.rinput || "",
      rows: 5,
    });
    this.codeArea = codeField.area;
    this.ainputArea = ainputField.area;
    this.rinputArea = rinputField.area;
  }

  buildOutputAreas() {
    this.finalOutputBox = createOutputBox(this.boundaryActionsRow, {
      title: "FOutput",
      classNames: ["wm-output-final-box", "wm-boundary-foutput"],
      bodyClassNames: ["wm-output-final"],
    });
    this.finalOutputEl = this.finalOutputBox.body;

    this.stateContainer = document.createElement("div");
    this.stateContainer.className = "wm-state wm-snapshot-view";
    this.statePanel.body.appendChild(this.stateContainer);

    this.runtimeOutputBox = createOutputBox(this.transitionActionsRow, {
      title: "ROutput",
      classNames: ["wm-output-runtime-box", "wm-transition-routput"],
      bodyClassNames: ["wm-output-runtime"],
    });
    this.runtimeOutputEl = this.runtimeOutputBox.body;
    this.createButton = null;
    this.stepButton = null;
    this.autoToggleButton = null;
    this.autoMarginInput = null;
  }

  buildRuntimeControls() {
    this.createControlEl = document.createElement("div");
    this.createControlEl.className = "wm-control wm-control-create wm-boundary-create";
    this.createButton = ensureChild(this.createControlEl, "button.wm-create", "button", "wm-create");
    this.createButton.textContent = "Create";
    this.createButton.addEventListener("click", () => {
      this.handleCreateClick().catch(err => console.error(err));
    });
    this.boundaryActionsRow.prepend(this.createControlEl);

    this.stepControlEl = document.createElement("div");
    this.stepControlEl.className = "wm-control wm-control-step wm-transition-controls";
    this.stepButton = ensureChild(this.stepControlEl, "button.wm-step", "button", "wm-step");
    this.autoToggleButton = ensureChild(this.stepControlEl, "button.wm-auto-toggle", "button", "wm-auto-toggle");
    this.autoMarginInput = ensureChild(this.stepControlEl, "input.wm-auto-margin", "input", "wm-auto-margin");
    this.stepButton.textContent = "Step";
    this.autoToggleButton.textContent = "Auto: Off";
    this.autoMarginInput.type = "number";
    this.autoMarginInput.step = "0.1";
    this.autoMarginInput.min = "0";
    if (!this.autoMarginInput.value) this.autoMarginInput.value = "0.5";
    this.autoMarginInput.placeholder = "auto margin (s)";
    this.autoMarginInput.setAttribute("aria-label", "auto margin (s)");
    this.stepButton.addEventListener("click", () => {
      this.stepFromCurrentInput({ triggeredByAuto: false }).catch(err => console.error(err));
    });
    this.autoToggleButton.addEventListener("click", () => this.toggleAutoStep());
    this.autoMarginInput.addEventListener("change", () => {
      if (this.autoEnabled) {
        this.stopAutoStep();
        this.startAutoStep();
      }
    });
    this.stepControlEl.append(this.stepButton, this.autoToggleButton, this.autoMarginInput);
    this.transitionActionsRow.prepend(this.stepControlEl);
    this.setAutoUI(false);
  }

  setOutputTransforms({ routput, foutput } = {}) {
    if (routput) this.transformROutput = routput;
    if (foutput) this.transformFOutput = foutput;
  }

  writeMessage(message, { error = false } = {}) {
    if (error) {
      this.reportError(message);
    } else {
      this.reportStatus(message);
    }
  }

  clearMessage() {
    this.writeMessage("");
  }

  async initRuntime() {
    if (!this.machineName) return;
    try {
      const rmod = await import("./renderer.js");
      const Renderer = rmod.Renderer;
      if (typeof Renderer !== "function") {
        throw new Error('Renderer not found (expected export "Renderer")');
      }
      this.renderer = new Renderer(this.stateContainer);
      if (typeof this.renderer.draw !== "function") {
        throw new Error("Renderer missing draw()");
      }
      this.wrapper = new MachineWrapper(this.machineName);
      await this.wrapper.init();
      this.machineState = "ready";
      this.clearMessage();
      return true;
    } catch (e) {
      this.machineState = "init_failed";
      this.writeMessage(`Init error (${this.machineName}): ${e}`, { error: true });
      this.setControlsDisabled(true);
      return false;
    }
  }

  drawSnapshot(snapshot) {
    if (this.renderer) this.renderer.draw(snapshot);
  }

  get code() {
    return this.codeArea.value;
  }

  set code(value) {
    this.codeArea.value = value ?? "";
  }

  get ainput() {
    return this.ainputArea.value;
  }

  set ainput(value) {
    this.ainputArea.value = value ?? "";
  }

  get rinput() {
    return this.rinputArea.value;
  }

  set rinput(value) {
    this.rinputArea.value = value ?? "";
  }

  async createFromCurrentInputs() {
    if (this.machineState === "init_failed") {
      this.writeMessage("(init failed; reload required)", { error: true });
      return false;
    }
    if (this.machineState === "uninitialized") {
      this.writeMessage("(runtime machine not initialized)", { error: true });
      return false;
    }
    this.stopAutoStep();
    this.clearOutputs();
    this.clearMessage();
    try {
      await this.wrapper.createMachine(this.code, this.ainput);
      this.machineState = "machine_setted";
      const state = await this.wrapper.currentState();
      this.drawSnapshot(state);
      return true;
    } catch (e) {
      this.machineState = "init_failed";
      this.writeMessage(`init_fail: ${e}`, { error: true });
      return false;
    }
  }

  makeStepFailure() {
    return STEP_FAILED_RESULT;
  }

  validateStepReady() {
    if (this.machineState === "init_failed") {
      this.writeMessage("(init failed; reload required)", { error: true });
      return false;
    }
    if (this.machineState !== "machine_setted") {
      this.writeMessage("(machine not initialized; run Create first)", { error: true });
      return false;
    }
    return true;
  }

  async handleContinueStepResult(stepResult) {
    const state = await this.wrapper.currentState();
    this.drawSnapshot(state);
    const outputTarget = stepResult.routput ?? "";
    const output = await this.transformROutput(outputTarget);
    this.setROutput(output);
    this.clearMessage();
    return { output, stepped: true, halted: false };
  }

  async handleHaltStepResult(stepResult) {
    this.drawSnapshot(stepResult.snapshot);
    const outputTarget = stepResult.foutput ?? "";
    const output = await this.transformFOutput(outputTarget);
    this.setFOutput(output);
    this.clearMessage();
    return { output, stepped: true, halted: true };
  }

  async stepFromCurrentInput({ triggeredByAuto = false } = {}) {
    if (!this.validateStepReady()) return this.makeStepFailure();
    try {
      const stepResult = await this.wrapper.stepMachine(this.rinput);
      if (!stepResult || typeof stepResult.kind !== "string") {
        throw new Error(`Invalid step result: ${JSON.stringify(stepResult)}`);
      }
      if (stepResult.kind === "continue") return this.handleContinueStepResult(stepResult);
      if (stepResult.kind === "halt") return this.handleHaltStepResult(stepResult);
      throw new Error(`Unknown step result kind: ${stepResult.kind}`);
    } catch (e) {
      this.writeMessage(`${e}`, { error: true });
      if (triggeredByAuto && this.autoEnabled) this.stopAutoStep();
      return this.makeStepFailure();
    }
  }

  clearOutputs() {
    this.runtimeOutputEl.textContent = "";
    this.finalOutputEl.textContent = "";
  }

  setROutput(output) {
    this.runtimeOutputEl.style.color = "";
    this.runtimeOutputEl.textContent = output ?? "";
  }

  setFOutput(output) {
    this.finalOutputEl.style.color = "";
    this.finalOutputEl.textContent = output ?? "";
  }

  setAutoUI(enabled) {
    if (!this.autoToggleButton) return;
    this.autoToggleButton.textContent = enabled ? "Auto: On" : "Auto: Off";
    this.autoToggleButton.setAttribute("aria-pressed", enabled ? "true" : "false");
  }

  getAutoIntervalMs() {
    if (!this.autoMarginInput) return 0;
    const value = parseFloat(this.autoMarginInput.value);
    if (Number.isFinite(value) && value > 0) return value * 1000;
    return 0;
  }

  setControlsDisabled(disabled) {
    if (this.createButton) this.createButton.disabled = disabled;
    if (this.stepButton) this.stepButton.disabled = disabled;
    if (this.autoToggleButton) this.autoToggleButton.disabled = disabled;
    if (this.autoMarginInput) this.autoMarginInput.disabled = disabled;
  }

  toggleAutoStep() {
    if (this.autoEnabled) {
      this.stopAutoStep();
    } else {
      this.startAutoStep();
    }
  }

  startAutoStep() {
    if (this.autoEnabled) return;
    const interval = this.getAutoIntervalMs();
    if (!interval) {
      this.setAutoUI(this.autoEnabled);
      return;
    }
    this.autoEnabled = true;
    this.setAutoUI(true);
    this.scheduleAutoStep(interval);
  }

  stopAutoStep() {
    this.autoEnabled = false;
    if (this.autoTimerId) {
      clearTimeout(this.autoTimerId);
      this.autoTimerId = null;
    }
    this.setAutoUI(false);
  }

  scheduleAutoStep(interval) {
    clearTimeout(this.autoTimerId);
    this.autoTimerId = window.setTimeout(() => {
      this.runAutoStep().catch(err => console.error(err));
    }, interval);
  }

  async runAutoStep() {
    if (!this.autoEnabled) return;
    const result = await this.stepFromCurrentInput({ triggeredByAuto: true });
    if (!this.autoEnabled) return;
    if (!result?.stepped || result.halted) {
      this.stopAutoStep();
      return;
    }
    const interval = this.getAutoIntervalMs();
    if (!interval) {
      this.stopAutoStep();
      return;
    }
    this.scheduleAutoStep(interval);
  }

  async handleCreateClick() {
    await this.createFromCurrentInputs();
  }
}

class CompilerEdgeView {
  constructor(host, {
    edgeMeta = null,
    onCompileCode,
    onCompileAInput,
    onCompileRInput,
    reportError = null,
  } = {}) {
    this.reportError = reportError || (() => { });
    this.edgeMeta = edgeMeta;
    this.wrapper = null;
    this.available = false;
    this.loadError = null;
    this.element = document.createElement("section");
    this.element.className = "wm-edge";
    const edgeTitle = document.createElement("div");
    edgeTitle.className = "wm-edge-title";
    edgeTitle.textContent = edgeMeta
      ? `Compiler (${edgeMeta.source.name} => ${edgeMeta.target.name})`
      : "Compiler";
    const edgeBody = document.createElement("div");
    edgeBody.className = "wm-edge-body";

    const compileStrip = document.createElement("div");
    compileStrip.className = "wm-compile-strip";
    this.compileButtons = {
      code: ensureChild(compileStrip, "button.wm-compile-code", "button", "wm-compile-code"),
      ainput: ensureChild(compileStrip, "button.wm-compile-ainput", "button", "wm-compile-ainput"),
      rinput: ensureChild(compileStrip, "button.wm-compile-rinput", "button", "wm-compile-rinput"),
    };
    this.compileButtons.code.textContent = "compile(Code)";
    this.compileButtons.ainput.textContent = "encode_ainput";
    this.compileButtons.rinput.textContent = "encode_rinput";
    if (onCompileCode) this.compileButtons.code.addEventListener("click", onCompileCode);
    if (onCompileAInput) this.compileButtons.ainput.addEventListener("click", onCompileAInput);
    if (onCompileRInput) this.compileButtons.rinput.addEventListener("click", onCompileRInput);

    this.statusPre = document.createElement("pre");
    this.statusPre.className = "wm-edge-status";
    edgeBody.append(compileStrip, this.statusPre);
    this.element.append(edgeTitle, edgeBody);
    host.appendChild(this.element);
  }

  setStatusLines(lines) {
    this.statusPre.textContent = (lines || []).join("\n");
  }

  setDisabled(disabled) {
    Object.values(this.compileButtons).forEach(btn => {
      btn.disabled = !!disabled;
    });
  }

  async initWrapper() {
    if (!this.edgeMeta) {
      this.refreshStatus();
      return false;
    }
    const edge = this.edgeMeta;
    try {
      this.wrapper = new CompilerWrapper(edge.compilerName);
      await this.wrapper.init();
      this.available = true;
      this.loadError = null;
    } catch (e) {
      this.available = false;
      this.loadError = e;
      this.wrapper = null;
      wmLog(
        `compiler unavailable for edge ${edge.source.name} => ${edge.target.name} (${edge.compilerName}):`,
        e,
      );
      this.reportError(`Compiler unavailable: ${edge.source.name} => ${edge.target.name}`);
    }
    this.refreshStatus();
    return this.available;
  }

  ensureAvailable() {
    if (!this.available || !this.wrapper) {
      const edge = this.edgeMeta;
      if (!edge) throw new Error("Compiler edge is not configured");
      throw new Error(`Compiler not available for edge ${edge.source.name} => ${edge.target.name}`);
    }
  }

  async encode(kind, value) {
    const encodeOp = COMPILER_ENCODE_OPS[kind];
    if (!encodeOp) throw new Error(`Unknown encode kind: ${kind}`);
    this.ensureAvailable();
    return encodeOp(this.wrapper)(value);
  }

  async decode(kind, value) {
    const decodeOp = COMPILER_DECODE_OPS[kind];
    if (!decodeOp) throw new Error(`Unknown decode kind: ${kind}`);
    this.ensureAvailable();
    return decodeOp(this.wrapper)(value);
  }

  refreshStatus() {
    if (!this.edgeMeta) {
      this.setStatusLines(["(no compiler edge configured)"]);
      this.setDisabled(true);
      return;
    }
    const edge = this.edgeMeta;
    if (this.available) {
      this.setStatusLines([`${edge.source.name} => ${edge.target.name}: ${edge.compilerName}`]);
      this.setDisabled(false);
      return;
    }
    const status = this.loadError
      ? `unavailable (${edge.compilerName})`
      : `pending (${edge.compilerName})`;
    this.setStatusLines([`${edge.source.name} => ${edge.target.name}: ${status}`]);
    this.setDisabled(true);
  }
}

// -------------------------------------
// PipelineController
// -------------------------------------
class PipelineController {
  constructor(root) {
    this.root = root;
    this.parseModelSpec();
    const defaults = this.readDefaults();
    this.buildViews(defaults);

    wmLog("model spec:", this.modelSpec, "pipeline:", this.pipelineNames.join(" => "));

  }

  parseModelSpec() {
    this.modelSpec = (this.root.dataset.model || "").trim();
    this.pipelineNames = parsePipelineSpec(this.modelSpec);
    if (this.pipelineNames.length === 0 && this.modelSpec) {
      this.pipelineNames = [this.modelSpec];
    }
    if (this.pipelineNames.length === 0) {
      this.pipelineNames = ["(missing-model)"];
    }
    this.edgeMetas = buildCompilerEdgeMetas(this.pipelineNames);
  }

  readDefaults() {
    return {
      code: extractPlainScript(this.root, "default-code"),
      ainput: extractPlainScript(this.root, "default-ainput"),
      rinput: extractPlainScript(this.root, "default-rinput"),
    };
  }

  async transferThroughCompiler(edgeView, kind, readValue, writeValue) {
    if (!edgeView) {
      this.writeControllerMessage("(pipeline compiler layer not available)", { error: true });
      return;
    }
    try {
      const value = await edgeView.encode(kind, readValue());
      writeValue(value);
      this.clearControllerMessage();
    } catch (e) {
      console.error(e);
      this.writeControllerMessage(`Compile error: ${e}`, { error: true });
    }
  }

  async decodeThroughEdges(kind, value) {
    let current = value;
    for (let i = this.edgeViews.length - 1; i >= 0; i -= 1) {
      current = await this.edgeViews[i].decode(kind, current);
    }
    return current;
  }

  buildNodeTitle(name, index, lastIndex) {
    if (lastIndex === 0) return `Machine (${name})`;
    if (index === 0) return `Source Machine (${name})`;
    if (index === lastIndex) return `Runtime Machine (${name})`;
    return `IR Machine ${index} (${name})`;
  }

  buildNodeRole(index, lastIndex) {
    if (lastIndex === 0) return "machine";
    if (index === 0) return "source";
    if (index === lastIndex) return "runtime";
    return "ir";
  }

  getNodeDefaults(defaults, index) {
    if (index === 0) return defaults;
    return { code: "", ainput: "", rinput: "" };
  }

  buildViews(defaults) {
    this.root.replaceChildren();
    this.controllerStatusPanel = createPanel(this.root, "wm-panel-controller-status", "Status / Error");
    this.controllerStatusBox = createOutputBox(this.controllerStatusPanel.body, {
      title: "PipelineController",
      classNames: ["wm-controller-status-box"],
      bodyClassNames: ["wm-status"],
    });
    this.controllerStatusEl = this.controllerStatusBox.body;

    this.pipelineRoot = ensureChild(this.root, ".wm-pipeline", "div", "wm-pipeline");
    this.pipelineRoot.classList.add("wm-pipeline-vertical");
    const reportStatus = message => this.setStatus(message);
    const reportError = message => this.setError(message);
    this.nodeViews = [];
    this.edgeViews = [];
    const lastIndex = this.pipelineNames.length - 1;

    for (let i = 0; i < this.pipelineNames.length; i += 1) {
      const nodeName = this.pipelineNames[i];
      const nodeView = new MachineNodeView(this.pipelineRoot, {
        title: this.buildNodeTitle(nodeName, i, lastIndex),
        role: this.buildNodeRole(i, lastIndex),
        machineName: nodeName,
        defaults: this.getNodeDefaults(defaults, i),
        runtimeControls: i === lastIndex,
        runtimeLabels: i === lastIndex && lastIndex > 0,
        reportStatus,
        reportError,
      });
      this.nodeViews.push(nodeView);

      if (i < lastIndex) {
        const edgeMeta = this.edgeMetas[i];
        const sourceNodeView = nodeView;
        let edgeView = null;
        edgeView = new CompilerEdgeView(this.pipelineRoot, {
          edgeMeta,
          onCompileCode: () => {
            const targetNodeView = this.nodeViews[i + 1];
            return this.transferThroughCompiler(edgeView, "code", () => sourceNodeView.code, value => { targetNodeView.code = value; });
          },
          onCompileAInput: () => {
            const targetNodeView = this.nodeViews[i + 1];
            return this.transferThroughCompiler(edgeView, "ainput", () => sourceNodeView.ainput, value => { targetNodeView.ainput = value; });
          },
          onCompileRInput: () => {
            const targetNodeView = this.nodeViews[i + 1];
            return this.transferThroughCompiler(edgeView, "rinput", () => sourceNodeView.rinput, value => { targetNodeView.rinput = value; });
          },
          reportError,
        });
        this.edgeViews.push(edgeView);
      }
    }

    this.sourceNodeView = this.nodeViews[0];
    this.runtimeNodeView = this.nodeViews[lastIndex];

    this.runtimeNodeView.setOutputTransforms({
      routput: async value => this.decodeThroughEdges("routput", value),
      foutput: async value => this.decodeThroughEdges("foutput", value),
    });
  }

  disableUI() {
    this.runtimeNodeView.setControlsDisabled(true);
    for (const edgeView of this.edgeViews) edgeView.setDisabled(true);
    this.runtimeNodeView.stopAutoStep();
  }

  async init() {
    try {
      for (const edgeView of this.edgeViews) {
        await edgeView.initWrapper();
      }

      const runtimeReady = await this.runtimeNodeView.initRuntime();
      if (!runtimeReady) {
        this.disableUI();
        return;
      }
      this.runtimeNodeView.clearOutputs();
      this.clearControllerMessage();
    } catch (e) {
      this.writeControllerMessage(`Init error: ${e}`, { error: true });
      this.disableUI();
      return;
    }
  }

  writeControllerMessage(message, { error = false } = {}) {
    if (!this.controllerStatusEl) return;
    this.controllerStatusEl.style.color = error ? "red" : "";
    this.controllerStatusEl.textContent = message ?? "";
  }

  clearControllerMessage() {
    this.writeControllerMessage("");
  }

  setStatus(message) {
    this.writeControllerMessage(message);
  }

  setError(message) {
    this.writeControllerMessage(message, { error: true });
  }

}

// -------------------------------------
// エントリポイント
// -------------------------------------
async function setupAllModels() {
  const roots = document.querySelectorAll("[data-model]");
  const tasks = [];

  roots.forEach(root => {
    const vm = new PipelineController(root);
    tasks.push(vm.init());
  });

  await Promise.all(tasks);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    setupAllModels().catch(err => console.error(err));
  });
} else {
  setupAllModels().catch(err => console.error(err));
}

console.log("script.js loaded");
