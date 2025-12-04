// assets/script.js
// このスクリプトは、ページ内の <div data-model="..."> を探して、
// それぞれに対応する wasm モジュール (./wasm_bundle/<model>.js) を動的に読み込み、
// 簡単な playground UI (textarea + button + output + canvas) を作って動かします。

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
// ViewModel: 1つの <div data-model="..."> に対応
//   - wasm_bundle/<model>.js を動的 import
//   - default() で wasm 初期化
//   - step_machine / current_machine / output_machine を呼ぶ
// -------------------------------------
class ViewModel {
  constructor(root) {
    this.root = root;
    this.modelName = root.dataset.model || "default";

    // default 値を div 内の <script type="text/plain"> から取得
    this.defaultInput = extractPlainScript(root, "default-input");
    this.defaultCode = extractPlainScript(root, "default-code");

    // UI 部品を用意（なければ作る）
    this.inputArea = ensureChild(root, "textarea.wm-input", "textarea", "wm-input");
    this.codeArea = ensureChild(root, "textarea.wm-code", "textarea", "wm-code");
    this.stepButton = ensureChild(root, "button.wm-step", "button", "wm-step");
    this.outputPre = ensureChild(root, "pre.wm-output", "pre", "wm-output");
    this.canvas = ensureChild(root, "canvas.wm-canvas", "canvas", "wm-canvas");

    this.ctx = this.canvas.getContext("2d");

    // ラベルが空ならデフォルト文字列
    if (!this.stepButton.textContent) {
      this.stepButton.textContent = "Step";
    }

    // textarea にデフォルト値をセット
    if (!this.inputArea.value) {
      this.inputArea.value = this.defaultInput;
    }
    if (!this.codeArea.value) {
      this.codeArea.value = this.defaultCode;
    }

    // wasm モジュール (glue JS) とその export 群
    this.module = null;
    this.api = null;

    // イベントハンドラ
    this.stepButton.addEventListener("click", () => {
      this.handleStepClick().catch(err => console.error(err));
    });
  }

  // wasm モジュール読み込み & 初期化
  async init() {
    // 1) wasm モジュール
    const wasmPath = `./wasm_bundle/${this.modelName}.js`;
    this.module = await import(wasmPath);
    if (typeof this.module.default === "function") {
      await this.module.default();
    }
    this.api = this.module;

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
  }

  // wasm 関数呼び出しのラッパ（関数が存在しない場合は何もしない）
  async callWasm(funcName, ...args) {
    if (!this.api || typeof this.api[funcName] !== "function") {
      return undefined;
    }
    // 同期関数でも Promise でも扱えるように Promise.resolve で包む
    return await Promise.resolve(this.api[funcName](...args));
  }

  async handleStepClick() {
    const inputStr = this.inputArea.value;

    try {
      const result = await this.callWasm("step_machine", inputStr);
      if (result !== undefined) {
        this.outputPre.textContent = JSON.stringify(result, null, 2);
      } else {
        this.outputPre.textContent = "(step_machine is not exported)";
      }
      await this.refreshView();
    } catch (e) {
      console.error(e);
      this.outputPre.textContent = `Error: ${e}`;
    }
  }

  async refreshView() {
    try {
      const state = await this.callWasm("current_machine");
      if (state !== undefined) {
        this.draw(state);
      }
    } catch (e) {
      console.error("current_machine failed:", e);
    }
  }

  // 状態の描画。とりあえずデフォルト実装はカウンタ用を想定。
  // モデルごとに変えたくなったら、modelName ごとに分岐したり、
  // 別の renderer レジストリを使う形に拡張すればOK。
  draw(state) {
    if (!this.ctx) return;

    const { ctx, canvas } = this;
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // 簡単な例: state が { count: number } を持つと仮定
    // （他のモデルでは自由に拡張・書き換えしてOK）
    const count = (state && typeof state.count === "number") ? state.count : 0;

    ctx.font = "20px sans-serif";
    ctx.textBaseline = "top";
    ctx.fillText(`${this.modelName}: count = ${count}`, 10, 10);
  }
}

// -------------------------------------
// エントリポイント: ページ内の <div data-model> すべてを初期化
// -------------------------------------
async function setupAllModels() {
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
