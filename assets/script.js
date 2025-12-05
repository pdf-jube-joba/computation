// assets/script.js
// このスクリプトは、ページ内の <div data-model="..."> を探して、
// それぞれに対応する wasm モジュール (./wasm_bundle/<model>.js) を動的に読み込み、
// 簡単な playground UI (textarea + button + output + canvas) を作って動かします。

// Renderer interface (JSDoc for documentation only):
// /**
//  * @callback Renderer
//  * @param {any} state   - current_machine() の結果
//  * @param {CanvasRenderingContext2D} ctx
//  * @param {HTMLCanvasElement} canvas
//  * @param {ViewModel} vm   - ViewModel インスタンス
//  */
//
// WASM module interface (feature共通):
// - default(): wasm-pack が生成する初期化関数
// - create(input: string): machine を初期化
// - step_machine(input: string): Step 実行
// - current_machine(): 現在状態を返す

// -------------------------------------
// ヘルパー: <script type="text/plain" class="..."> からデフォルト文字列を取得
// -------------------------------------
function extractPlainScript(root, className) {
  console.log("extractPlainScript");
  const el = root.querySelector(`script.${className}[type="text/plain"]`);
  if (!el) return "";
  return el.textContent || "";
}

// -------------------------------------
// ヘルパー: 子要素を確保 (なければ生成)
// -------------------------------------
function ensureChild(root, selector, tagName, className) {
  console.log("ensureChild");
  let el = root.querySelector(selector);
  if (!el) {
    el = document.createElement(tagName);
    if (className) el.classList.add(className);
    root.appendChild(el);
  }
  return el;
}

// -------------------------------------
// ViewModel: 1つの <div data-model="..."> に対応
//   - wasm_bundle/<model>.js を動的 import
//   - default() で wasm 初期化
//   - step_machine / current_machine / output_machine を呼ぶ
// -------------------------------------
class ViewModel {
  constructor(root) {
    console.log("ViewModel.constructor");
    this.root = root;
    this.modelName = root.dataset.model || "default";

    // default 値を div 内の <script type="text/plain"> から取得
    this.defaultInput = extractPlainScript(root, "default-input");
    this.defaultCode = extractPlainScript(root, "default-code");

    // UI 部品を用意（なければ作る）
    this.codeLabel = ensureChild(root, ".wm-code-label", "div", "wm-code-label");
    this.inputLabel = ensureChild(root, ".wm-input-label", "div", "wm-input-label");
    this.codeArea = ensureChild(root, "textarea.wm-code", "textarea", "wm-code");
    this.inputArea = ensureChild(root, "textarea.wm-input", "textarea", "wm-input");
    this.createButton = ensureChild(root, "button.wm-create", "button", "wm-create");
    this.stepButton = ensureChild(root, "button.wm-step", "button", "wm-step");
    this.outputPre = ensureChild(root, "pre.wm-output", "pre", "wm-output");
    this.canvas = ensureChild(root, "canvas.wm-canvas", "canvas", "wm-canvas");

    // ラベルテキスト
    this.codeLabel.textContent = "code";
    this.inputLabel.textContent = "input";

    this.ctx = this.canvas.getContext("2d");

    // ラベルが空ならデフォルト文字列
    if (!this.createButton.textContent) {
      this.createButton.textContent = "Create";
    }
    if (!this.stepButton.textContent) {
      this.stepButton.textContent = "Step";
    }

    // textarea にデフォルト値をセット
    if (this.defaultInput) {
      this.inputArea.value = this.defaultInput;
    } else if (!this.inputArea.value) {
      this.inputArea.value = "";
    }
    if (this.defaultCode) {
      this.codeArea.value = this.defaultCode;
    } else if (!this.codeArea.value) {
      this.codeArea.value = "";
    }

    // 並び順を固定（code -> input -> button -> output -> canvas）
    root.append(this.codeLabel, this.codeArea, this.createButton, this.inputLabel, this.inputArea, this.stepButton, this.outputPre, this.canvas);

    // wasm モジュール (glue JS) とその export 群
    this.module = null;
    this.api = null;
    this.createFn = null;
    this.stepFn = null;
    this.currentFn = null;
    this.initialized = false;

    // イベントハンドラ
    this.createButton.addEventListener("click", () => {
      this.handleCreateClick().catch(err => console.error(err));
    });
    this.stepButton.addEventListener("click", () => {
      this.handleStepClick().catch(err => console.error(err));
    });
  }

  // wasm モジュール読み込み & 初期化
  async init() {
    console.log("ViewModel.init");

    // 1) wasm モジュール
    const wasmPath = `./wasm_bundle/${this.modelName}.js`;
    this.module = await import(wasmPath);
    this.api = this.module;
    this.createFn = typeof this.api.create === "function" ? this.api.create : null;
    this.stepFn = typeof this.api.step_machine === "function" ? this.api.step_machine : null;
    this.currentFn = typeof this.api.current_machine === "function" ? this.api.current_machine : null;
    // wasm-pack で生成された glue の default() は初期化に必要。
    if (typeof this.module.default === "function") {
      await this.module.default();
    }
    // 各 feature 共通の API: create / step_machine / current_machine
    // codeArea にデフォルト文字列があるときだけ初期化を行う
    if (this.codeArea.value.trim()) {
      if (this.createFn) {
        const initial = this.defaultInput || this.inputArea.value || "";
        await Promise.resolve(this.createFn(initial));
        this.initialized = true;
      } else {
        console.warn(`WASM module for ${this.modelName} does not expose create()`);
      }
    }

    // 2) renderer モジュール
    const rendererPath = `./renderers/${this.modelName}.js`;
    try {
      const rmod = await import(rendererPath);
      // default export or named export 'render'
      this.renderer = rmod.default || rmod.render || null;
    } catch (e) {
      console.warn(`No renderer for model "${this.modelName}"`, e);
      this.renderer = null;
    }

    // renderer がなければデフォルト renderer を読み込む
    if (!this.renderer) {
      try {
        const def = await import("./renderers/default.js");
        this.renderer = def.default || def.render || null;
      } catch (e) {
        console.warn("No default renderer found", e);
      }
    }

    if (this.initialized) {
      await this.refreshView();
    }
  }

  async handleCreateClick() {
    console.log("ViewModel.handleCreateClick");
    if (!this.createFn) {
      this.outputPre.textContent = "(create is not exported)";
      return;
    }
    try {
      const codeStr = this.codeArea.value;
      console.log("Creating machine with code:", codeStr);
      await Promise.resolve(this.createFn(codeStr));
      this.initialized = true;
      await this.refreshView();
    } catch (e) {
      console.error(e);
      this.outputPre.textContent = `Error: ${e}`;
    }
  }

  async handleStepClick() {
    console.log("ViewModel.handleStepClick");
    if (!this.stepFn) {
      this.outputPre.textContent = "(step_machine is not exported)";
      return;
    }
    if (!this.initialized) {
      this.outputPre.textContent = "(machine not initialized; run Create first)";
      return;
    }

    try {
      const inputStr = this.inputArea.value;
      console.log("Stepping machine with input:", inputStr);
      const result = await Promise.resolve(this.stepFn(inputStr));
      const display = result === undefined ? "(no result)" : JSON.stringify(result, null, 2);
      this.outputPre.textContent = display;
      await this.refreshView();
    } catch (e) {
      console.error(e);
      this.outputPre.textContent = `Error: ${e}`;
    }
  }

  async refreshView() {
    console.log("ViewModel.refreshView");
    if (!this.initialized) return;
    try {
      if (!this.currentFn) return;
      const state = await Promise.resolve(this.currentFn());
      this.draw(state);
    } catch (e) {
      console.error("current_machine failed:", e);
    }
  }

  draw(state) {
    console.log("ViewModel.draw");
    if (!this.ctx) return;

    const { ctx, canvas } = this;
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    if (typeof this.renderer !== "function") {
      throw new Error(`Renderer is missing for model "${this.modelName}"`);
    }

    this.renderer(state, ctx, canvas, this);
  }
}

// -------------------------------------
// エントリポイント: ページ内の <div data-model> すべてを初期化
// -------------------------------------
async function setupAllModels() {
  console.log("setupAllModels");
  const roots = document.querySelectorAll("[data-model]");
  const tasks = [];

  roots.forEach(root => {
    const vm = new ViewModel(root);
    tasks.push(vm.init());
  });

  // まとめて await（個々のエラーは catch でログが出る）
  await Promise.all(tasks);
}

// DOMContentLoaded 後に実行
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    setupAllModels().catch(err => console.error(err));
  });
} else {
  setupAllModels().catch(err => console.error(err));
}
