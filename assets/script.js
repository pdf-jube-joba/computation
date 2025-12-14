// assets/script.js
// モデルごとの wasm バンドルとレンダラーを読み込み、簡単な UI を構築する。

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
  constructor(root, { defaultCode = "", defaultAInput = "", defaultRInput = "" } = {}) {
    this.root = root;
    this.codeLabel = ensureChild(root, ".wm-code-label", "div", "wm-code-label");
    this.ainputLabel = ensureChild(root, ".wm-ainput-label", "div", "wm-ainput-label");
    this.rInputLabel = ensureChild(root, ".wm-input-label", "div", "wm-input-label");
    this.codeArea = ensureChild(root, "textarea.wm-code", "textarea", "wm-code");
    this.ainputArea = ensureChild(root, "textarea.wm-ainput", "textarea", "wm-ainput");
    this.rinputArea = ensureChild(root, "textarea.wm-input", "textarea", "wm-input");

    this.codeLabel.textContent = "code";
    this.ainputLabel.textContent = "ahead-of-time input";
    this.rInputLabel.textContent = "runtime input";

    if (defaultCode && !this.codeArea.value) this.codeArea.value = defaultCode;
    if (defaultAInput && !this.ainputArea.value) this.ainputArea.value = defaultAInput;
    if (defaultRInput && !this.rinputArea.value) this.rinputArea.value = defaultRInput;

    root.append(this.codeLabel, this.codeArea, this.ainputLabel, this.ainputArea, this.rInputLabel, this.rinputArea);
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

    this.createButton = ensureChild(root, "button.wm-create", "button", "wm-create");
    this.stepButton = ensureChild(root, "button.wm-step", "button", "wm-step");
    this.autoToggleButton = ensureChild(root, "button.wm-auto-toggle", "button", "wm-auto-toggle");
    this.autoMarginInput = ensureChild(root, "input.wm-auto-margin", "input", "wm-auto-margin");

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
    root.append(this.createButton, this.stepButton, this.autoToggleButton, this.autoMarginInput);
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
      throw new Error(`WASM module is missing exports: ${missing.join(", ")}`);
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

    this.textareas = new TextAreaTriple(root, {
      defaultCode,
      defaultAInput,
      defaultRInput,
    });
    this.targetTextareas = this.isCompiler
      ? new TextAreaTriple(root, { defaultCode: "", defaultAInput: "", defaultRInput: "" })
      : null;
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
  }

  async initializeMachine(codeStr, ainputStr) {
    if (this.status === "init_failed") return false;
    if (this.status === "uninitialized") {
      this.outputPre.textContent = "(init not completed)";
      return false;
    }
    try {
      await this.machine.createMachine(codeStr, ainputStr);
      this.status = "machine_setted";
      return true;
    } catch (e) {
      this.status = "init_failed";
      console.error("initializeMachine failed:", e);
      this.outputPre.textContent = `Error: ${e}`;
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
    } catch (e) {
      console.error("init failed:", e);
      this.outputPre.textContent = `Init error: ${e}`;
      this.disableUI();
      this.status = "init_failed";
      return;
    }
  }

  async handleCreateClick() {
    this.control.stopAuto();
    if (this.status === "init_failed") {
      this.outputPre.textContent = "(init failed; reload required)";
      return;
    }
    if (this.status === "uninitialized") {
      this.outputPre.textContent = "(init not completed)";
      return;
    }
    this.outputPre.textContent = "";

    try {
      if (this.isCompiler) {
        const srcCode = this.textareas.code;
        const srcAInput = this.textareas.ainput;
        const compiledCode = await this.compiler.compileCode(srcCode);
        const compiledAInput = await this.compiler.compileAInput(srcAInput);
        this.targetTextareas.code = compiledCode;
        this.targetTextareas.ainput = compiledAInput;
        await this.machine.createMachine(compiledCode, compiledAInput);
        this.status = "machine_setted";
        const state = await this.machine.currentState();
        this.draw(state);
      } else {
        const codeStr = this.textareas.code;
        const ainputStr = this.textareas.ainput;
        const ok = await this.initializeMachine(codeStr, ainputStr);
        if (ok) {
          const state = await this.machine.currentState();
          this.draw(state);
        }
      }
    } catch (e) {
      console.error(e);
      this.outputPre.textContent = `Error: ${e}`;
    }
  }

  async handleStepClick({ triggeredByAuto = false } = {}) {
    if (this.status === "init_failed") {
      this.outputPre.textContent = "(init failed; reload required)";
      return { output: undefined, stepped: false };
    }
    if (this.status !== "machine_setted") {
      this.outputPre.textContent = "(machine not initialized; run Create first)";
      return { output: undefined, stepped: false };
    }

    try {
      if (this.isCompiler) {
        const srcRInput = this.textareas.rinput;
        const compiledRInput = await this.compiler.compileRInput(srcRInput);
        this.targetTextareas.rinput = compiledRInput;
        const outputTarget = await this.machine.stepMachine(compiledRInput);
        const outputSource = outputTarget ? await this.compiler.decodeOutput(outputTarget) : "";
        const state = await this.machine.currentState();
        this.draw(state, outputSource);
        return { output: outputSource, stepped: true };
      } else {
        const runtimeInputStr = this.textareas.rinput;
        const output = await this.machine.stepMachine(runtimeInputStr);
        const state = await this.machine.currentState();
        this.draw(state, output);
        return { output, stepped: true };
      }
    } catch (e) {
      console.error(e);
      this.outputPre.textContent = `Error: ${e}`;
      if (triggeredByAuto && this.control.autoEnabled) {
        this.control.stopAuto();
      }
      return { output: undefined, stepped: false };
    }
  }

  draw(state, output) {
    this.renderer.draw(state);
    if (output !== undefined) {
      this.outputPre.textContent = output;
    } else {
      this.outputPre.textContent = "";
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
