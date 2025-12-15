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

// -------------------------------------
// TextAreaTriple: code / ahead-of-time / runtime をまとめて扱う
// -------------------------------------
class TextAreaTriple {
  constructor(host, { defaultCode = "", defaultAInput = "", defaultRInput = "" } = {}) {
    this.host = host;
    const grid = ensureChild(host, ".wm-form-grid", "div", "wm-form-grid");

    // code
    const codeField = ensureChild(grid, ".wm-field-code", "div", "wm-field");
    const codeLabel = ensureChild(codeField, "label.wm-code-label", "label", "wm-code-label");
    codeLabel.textContent = "code";
    const codeArea = ensureChild(codeField, "textarea.wm-code", "textarea", "wm-code");

    // ahead-of-time input
    const ainputField = ensureChild(grid, ".wm-field-ainput", "div", "wm-field");
    const ainputLabel = ensureChild(ainputField, "label.wm-ainput-label", "label", "wm-ainput-label");
    ainputLabel.textContent = "ahead-of-time input";
    const ainputArea = ensureChild(ainputField, "textarea.wm-ainput", "textarea", "wm-ainput");

    // runtime input
    const rinputField = ensureChild(grid, ".wm-field-rinput", "div", "wm-field");
    const rInputLabel = ensureChild(rinputField, "label.wm-input-label", "label", "wm-input-label");
    rInputLabel.textContent = "runtime input";
    const rinputArea = ensureChild(rinputField, "textarea.wm-input", "textarea", "wm-input");

    if (defaultCode && !codeArea.value) codeArea.value = defaultCode;
    if (defaultAInput && !ainputArea.value) ainputArea.value = defaultAInput;
    if (defaultRInput && !rinputArea.value) rinputArea.value = defaultRInput;

    this.codeArea = codeArea;
    this.ainputArea = ainputArea;
    this.rinputArea = rinputArea;
  }

  get code() {
    return this.codeArea.value.trim();
  }

  set code(value) {
    this.codeArea.value = value ?? "";
  }

  get ainput() {
    return this.ainputArea.value.trim();
  }

  set ainput(value) {
    this.ainputArea.value = value ?? "";
  }

  get rinput() {
    return this.rinputArea.value.trim();
  }

  set rinput(value) {
    this.rinputArea.value = value ?? "";
  }
}

// -------------------------------------
// Control: create/step/auto 周りの UI とタイマー制御をまとめる
// -------------------------------------
class Control {
  constructor(root, { onCreate, onStep }) {
    this.onCreate = onCreate;
    this.onStep = onStep;

    const controlContainer = ensureChild(root, ".wm-control", "div", "wm-control");

    this.createButton = ensureChild(controlContainer, "button.wm-create", "button", "wm-create");
    this.stepButton = ensureChild(controlContainer, "button.wm-step", "button", "wm-step");
    this.autoToggleButton = ensureChild(controlContainer, "button.wm-auto-toggle", "button", "wm-auto-toggle");
    this.autoMarginInput = ensureChild(controlContainer, "input.wm-auto-margin", "input", "wm-auto-margin");

    if (!this.createButton.textContent) this.createButton.textContent = "Create";
    if (!this.stepButton.textContent) this.stepButton.textContent = "Step";
    if (!this.autoToggleButton.textContent) this.autoToggleButton.textContent = "Auto: Off";

    this.autoMarginInput.type = "number";
    this.autoMarginInput.step = "0.1";
    this.autoMarginInput.min = "0";
    if (!this.autoMarginInput.value) this.autoMarginInput.value = "0.5";
    this.autoMarginInput.placeholder = "auto margin (s)";
    this.autoMarginInput.setAttribute("aria-label", "auto margin (s)");

    this.autoEnabled = false;
    this.autoTimerId = null;

    this.createButton.addEventListener("click", () => this.onCreate());
    this.stepButton.addEventListener("click", () => this.onStep({ triggeredByAuto: false }));
    this.autoToggleButton.addEventListener("click", () => this.toggleAuto());
    this.autoMarginInput.addEventListener("change", () => {
      if (this.autoEnabled) {
        this.stopAuto();
        this.startAuto();
      }
    });

    this.updateAutoUI();
    controlContainer.append(this.createButton, this.stepButton, this.autoToggleButton, this.autoMarginInput);
  }

  disable() {
    this.createButton.disabled = true;
    this.stepButton.disabled = true;
    this.autoToggleButton.disabled = true;
    this.autoMarginInput.disabled = true;
    this.stopAuto();
  }

  updateAutoUI() {
    this.autoToggleButton.textContent = this.autoEnabled ? "Auto: On" : "Auto: Off";
    this.autoToggleButton.setAttribute("aria-pressed", this.autoEnabled ? "true" : "false");
  }

  getAutoIntervalMs() {
    const value = parseFloat(this.autoMarginInput.value);
    if (Number.isFinite(value) && value > 0) {
      return value * 1000;
    }
    return 0;
  }

  toggleAuto() {
    if (this.autoEnabled) {
      this.stopAuto();
    } else {
      this.startAuto();
    }
  }

  startAuto() {
    if (this.autoEnabled) return;
    const interval = this.getAutoIntervalMs();
    if (!interval) {
      this.updateAutoUI();
      return;
    }
    this.autoEnabled = true;
    this.updateAutoUI();
    this.scheduleNext(interval);
  }

  stopAuto() {
    this.autoEnabled = false;
    if (this.autoTimerId) {
      clearTimeout(this.autoTimerId);
      this.autoTimerId = null;
    }
    this.updateAutoUI();
  }

  scheduleNext(interval) {
    clearTimeout(this.autoTimerId);
    this.autoTimerId = window.setTimeout(() => {
      this.runAutoStep().catch(err => console.error(err));
    }, interval);
  }

  async runAutoStep() {
    if (!this.autoEnabled) return;
    const result = await this.onStep({ triggeredByAuto: true });
    if (!this.autoEnabled) return;
    if (!result?.stepped || result.output !== undefined) {
      this.stopAuto();
      return;
    }
    const interval = this.getAutoIntervalMs();
    if (!interval) {
      this.stopAuto();
      return;
    }
    this.scheduleNext(interval);
  }
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
    this.decodeOutputFn = null;
  }

  async init() {
    if (this.module) return;
    const wasmPath = `./wasm_bundle/${this.compilerName}.js?instance=${this.instanceId}`;
    const module = await import(wasmPath);
    this.module = module;
    this.compileCodeFn = typeof module.compile_code === "function" ? module.compile_code : null;
    this.compileAInputFn = typeof module.compile_ainput === "function" ? module.compile_ainput : null;
    this.compileRInputFn = typeof module.compile_rinput === "function" ? module.compile_rinput : null;
    this.decodeOutputFn = typeof module.decode_output === "function" ? module.decode_output : null;
    const missing = [];
    if (!this.compileCodeFn) missing.push("compile_code");
    if (!this.compileAInputFn) missing.push("compile_ainput");
    if (!this.compileRInputFn) missing.push("compile_rinput");
    if (!this.decodeOutputFn) missing.push("decode_output");
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

  async decodeOutput(output) {
    return Promise.resolve(this.decodeOutputFn(output));
  }
}

// -------------------------------------
// RendererWrapper: renderer をロードして SnapshotRenderer を得る
// -------------------------------------
class RendererWrapper {
  constructor(modelName, stateContainer) {
    this.modelName = modelName;
    this.stateContainer = stateContainer;
    this.snapshotRenderer = null;
  }

  async init() {
    const rendererPath = `./renderers/${this.modelName}.js`;
    const rmod = await import(rendererPath);
    const SnapshotRenderer = rmod.SnapshotRenderer;
    if (typeof SnapshotRenderer !== "function") {
      throw new Error(`SnapshotRenderer not found for model "${this.modelName}"`);
    }
    this.snapshotRenderer = new SnapshotRenderer(this.stateContainer);
    if (typeof this.snapshotRenderer.draw !== "function") {
      throw new Error(`SnapshotRenderer missing draw() for model "${this.modelName}"`);
    }
  }

  draw(state) {
    this.snapshotRenderer.draw(state);
  }
}

// -------------------------------------
// ViewModel
// -------------------------------------
class ViewModel {
  constructor(root) {
    this.root = root;
    this.modelName = root.dataset.model;
    this.isCompiler = this.modelName.includes("-");
    this.compilerName = this.isCompiler ? this.modelName : null;

    const defaultRInput = extractPlainScript(root, "default-rinput");
    const defaultAInput = extractPlainScript(root, "default-ainput");
    const defaultCode = extractPlainScript(root, "default-code");

    this.outputPre = ensureChild(root, "pre.wm-output", "pre", "wm-output");
    this.stateContainer = ensureChild(root, ".wm-state", "div", "wm-state");

    // Source inputs: always present for readability and consistency
    const sourceContainer = ensureChild(root, ".wm-source", "div", "wm-source");
    const sourceHeading = ensureChild(
      sourceContainer,
      ".wm-source-heading",
      "div",
      "wm-source-heading",
    );
    sourceHeading.textContent = this.isCompiler ? "Source (compile input)" : "Source";
    const sourceTextareas = new TextAreaTriple(sourceContainer, {
      defaultCode,
      defaultAInput,
      defaultRInput,
    });
    this.sourceInputs = sourceTextareas;

    // Machine/target side container (used for compiled outputs or direct execution)
    this.machineContainer = ensureChild(root, ".wm-machine", "div", "wm-machine");

    // by default, targetInputs points to sourceInputs
    this.targetInputs = this.sourceInputs;

    if (this.isCompiler) {
      // compile buttons per input kind
      const compileStrip = ensureChild(sourceContainer, ".wm-compile-strip", "div", "wm-compile-strip");
      this.compileButtons = {
        code: ensureChild(compileStrip, "button.wm-compile-code", "button", "wm-compile-code"),
        ainput: ensureChild(compileStrip, "button.wm-compile-ainput", "button", "wm-compile-ainput"),
        rinput: ensureChild(compileStrip, "button.wm-compile-rinput", "button", "wm-compile-rinput"),
      };
      this.compileButtons.code.textContent = "Compile code → target";
      this.compileButtons.ainput.textContent = "Compile AInput → target";
      this.compileButtons.rinput.textContent = "Compile RInput → target";
      this.compileButtons.code.addEventListener("click", () => this.runCompile("code"));
      this.compileButtons.ainput.addEventListener("click", () => this.runCompile("ainput"));
      this.compileButtons.rinput.addEventListener("click", () => this.runCompile("rinput"));

      const targetHeading = ensureChild(
        this.machineContainer,
        ".wm-target-heading",
        "div",
        "wm-target-heading",
      );
      targetHeading.textContent = "Compiled target";
      const targetTextareas = new TextAreaTriple(this.machineContainer, {
        defaultCode: "",
        defaultAInput: "",
        defaultRInput: "",
      });
      this.targetInputs = targetTextareas;
    }
    this.control = new Control(root, {
      onCreate: () => {
        this.handleCreateClick().catch(err => console.error(err));
      },
      onStep: opts => this.handleStepClick(opts),
    });

    root.append(this.outputPre, this.stateContainer);

    this.machine = null;
    this.compiler = null;
    this.status = "uninitialized"; // "ready" | "machine_setted" | "init_failed"
    this.targetModelName = this.isCompiler ? this.modelName.split("-").slice(-1)[0] : this.modelName;
    this.renderer = new RendererWrapper(this.targetModelName, this.stateContainer);

    console.log(`ViewModel for model "${this.modelName}" isCompiler: "${this.isCompiler}"`);

  }

  async initializeMachine(codeStr, ainputStr) {
    if (this.status === "init_failed") return false;
    if (this.status === "uninitialized") {
      this.setError("(init not completed)");
      return false;
    }
    try {
      await this.machine.createMachine(codeStr, ainputStr);
      this.status = "machine_setted";
      return true;
    } catch (e) {
      this.status = "init_failed";
      this.setError(`init_fail: ${e}`);
      return false;
    }
  }

  disableUI() {
    this.control.disable();
  }

  async init() {
    try {
      await this.renderer.init();

      if (this.isCompiler) {
        this.compiler = new CompilerWrapper(this.compilerName);
        await this.compiler.init();
        this.machine = new MachineWrapper(this.targetModelName);
        await this.machine.init();
      } else {
        this.machine = new MachineWrapper(this.modelName);
        await this.machine.init();
      }

      this.status = "ready";
      this.setOutput("");
    } catch (e) {
      this.setError(`Init error: ${e}`);
      this.disableUI();
      this.status = "init_failed";
      return;
    }
  }

  async handleCreateClick() {
    this.control.stopAuto();
    if (this.status === "init_failed") {
      this.setError("(init failed; reload required)");
      return;
    }
    if (this.status === "uninitialized") {
      this.setError("(init not completed)");
      return;
    }
    this.setOutput("");

    try {
      const codeStr = this.targetInputs.code;
      const ainputStr = this.targetInputs.ainput;
      const ok = await this.initializeMachine(codeStr, ainputStr);
      if (ok) {
        const state = await this.machine.currentState();
        this.renderer.draw(state);
      }
    } catch (e) {
      this.setOutput(`${e}`);
    }
  }

  async handleStepClick({ triggeredByAuto = false } = {}) {
    if (this.status === "init_failed") {
      this.setOutput("(init failed; reload required)");
      return { output: undefined, stepped: false };
    }
    if (this.status !== "machine_setted") {
      this.setOutput("(machine not initialized; run Create first)");
      return { output: undefined, stepped: false };
    }

    try {
      const runtimeInputStr = this.targetInputs.rinput;
      const outputTarget = await this.machine.stepMachine(runtimeInputStr);
      const state = await this.machine.currentState();
      this.renderer.draw(state);
      const decodedOutput =
        this.compiler && outputTarget ? await this.compiler.decodeOutput(outputTarget) : outputTarget;
      this.setOutput(decodedOutput === undefined ? "" : decodedOutput);
      return { output: decodedOutput, stepped: true };
    } catch (e) {
      this.setError(`${e}`);
      // Even on step error, try to render the latest state.
      if (triggeredByAuto && this.control.autoEnabled) {
        this.control.stopAuto();
      }
      return { output: undefined, stepped: false };
    }
  }

  setOutput(output) {
    this.outputPre.style.color = "";
    this.outputPre.textContent = output ?? "";
  }

  setError(message) {
    this.outputPre.style.color = "red";
    this.outputPre.textContent = message ?? "";
  }

  async runCompile(kind) {
    if (!this.isCompiler || !this.compiler) {
      this.setError("(compiler not available)");
      return;
    }
    if (this.status === "init_failed") {
      this.setError("(init failed; reload required)");
      return;
    }
    if (this.status === "uninitialized") {
      this.setError("(init not completed)");
      return;
    }
    try {
      if (kind === "code") {
        const compiled = await this.compiler.compileCode(this.sourceInputs.code);
        this.targetInputs.code = compiled;
      } else if (kind === "ainput") {
        const compiled = await this.compiler.compileAInput(this.sourceInputs.ainput);
        this.targetInputs.ainput = compiled;
      } else if (kind === "rinput") {
        const compiled = await this.compiler.compileRInput(this.sourceInputs.rinput);
        this.targetInputs.rinput = compiled;
      }
    } catch (e) {
      console.error(e);
      this.setError(`Compile error: ${e}`);
    }
  }
}

// -------------------------------------
// エントリポイント
// -------------------------------------
async function setupAllModels() {
  const roots = document.querySelectorAll("[data-model]");
  const tasks = [];

  roots.forEach(root => {
    const vm = new ViewModel(root);
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
