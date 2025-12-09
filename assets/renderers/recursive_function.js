// assets/renderers/recursive_function.js
// Renderer for recursive_function machines: shows source code, tuple input hints, and the current Process tree.

export class Renderer {
  constructor(vm, stateContainer, outputContainer) {
    this.vm = vm;
    this.stateContainer = stateContainer;
    this.outputContainer = outputContainer;

    this.stateContainer.replaceChildren();
    this.outputContainer.replaceChildren();

    this.statusLine = document.createElement("div");
    this.statusLine.className = "rf-status";

    const codeSection = document.createElement("section");
    codeSection.className = "rf-section";
    const codeHeading = document.createElement("h4");
    codeHeading.textContent = "Function Definition";
    this.codeBlock = document.createElement("pre");
    this.codeBlock.className = "rf-code";
    codeSection.append(codeHeading, this.codeBlock);

    this.tupleLine = document.createElement("div");
    this.tupleLine.className = "rf-tuple";

    const processSection = document.createElement("section");
    processSection.className = "rf-section";
    const processHeading = document.createElement("h4");
    processHeading.textContent = "Current Process";
    this.processContainer = document.createElement("div");
    this.processContainer.className = "rf-process";
    processSection.append(processHeading, this.processContainer);

    this.stateContainer.append(this.statusLine, codeSection, this.tupleLine, processSection);

    this.outputMessage = document.createElement("div");
    this.outputMessage.className = "rf-output";
    this.outputContainer.appendChild(this.outputMessage);
  }

  drawState(state) {
    this.renderStatus(state);
    this.renderCode();
    this.renderTupleHint();
    this.renderProcess(state);
  }

  drawOutput(output) {
    if (!this.outputMessage) return;
    if (output === undefined) {
      this.outputMessage.textContent = "";
      return;
    }
    if (output === true) {
      this.outputMessage.textContent = "Process finished.";
    } else if (output === false) {
      this.outputMessage.textContent = "Process is still running.";
    } else {
      this.outputMessage.textContent = `output: ${this.safeStringify(output)}`;
    }
  }

  renderStatus(state) {
    if (!this.statusLine) return;
    if (!state) {
      this.statusLine.textContent = "state: (not initialized)";
      return;
    }
    const variant = this.extractVariant(state);
    const tag = variant?.tag ?? "?";
    this.statusLine.textContent =
      tag === "Result" ? "status: terminated" : `status: evaluating (${tag})`;
  }

  renderCode() {
    if (!this.codeBlock) return;
    const src = this.vm?.codeArea?.value ?? "";
    this.codeBlock.textContent = src.trim() ? src : "(no code loaded)";
  }

  renderTupleHint() {
    if (!this.tupleLine) return;
    const tupleText = this.vm?.inputArea?.value ?? "";
    if (!tupleText.trim()) {
      this.tupleLine.textContent =
        'Input: enter a tuple like "(0, 1, 2)" to reset, leave empty to step.';
    } else {
      this.tupleLine.textContent = `Input tuple: ${tupleText.trim()}`;
    }
  }

  renderProcess(state) {
    if (!this.processContainer) return;
    this.processContainer.replaceChildren();
    if (!state) {
      this.processContainer.textContent = "(no process)";
      return;
    }
    this.processContainer.appendChild(this.renderProcessNode(state));
  }

  renderProcessNode(node) {
    const variant = this.extractVariant(node);
    if (!variant) {
      return this.makeTextNode(this.safeStringify(node ?? "(unknown)"));
    }
    const { tag, value } = variant;
    switch (tag) {
      case "Result":
        return this.makeLeaf(`Result: ${this.formatNumber(value)}`);
      case "Comp":
        return this.renderCompNode(value);
      case "MuOpComp":
        return this.renderMuNode(value);
      default:
        return this.makeLeaf(`[${tag}] ${this.safeStringify(value)}`);
    }
  }

  renderCompNode(value) {
    const wrapper = document.createElement("div");
    wrapper.className = "rf-comp";
    const title = document.createElement("div");
    title.className = "rf-comp-title";
    title.textContent = `Call ${this.describeFunction(value?.function)} (${(value?.args || []).length} args)`;
    wrapper.appendChild(title);

    if (Array.isArray(value?.args) && value.args.length > 0) {
      const list = document.createElement("ol");
      list.className = "rf-args";
      value.args.forEach((arg, idx) => {
        const li = document.createElement("li");
        li.setAttribute("aria-label", `arg ${idx}`);
        li.appendChild(this.renderProcessNode(arg));
        list.appendChild(li);
      });
      wrapper.appendChild(list);
    } else {
      const emptyNote = document.createElement("div");
      emptyNote.className = "rf-args-empty";
      emptyNote.textContent = "(no arguments)";
      wrapper.appendChild(emptyNote);
    }
    return wrapper;
  }

  renderMuNode(value) {
    const wrapper = document.createElement("div");
    wrapper.className = "rf-mu";
    const headline = document.createElement("div");
    headline.className = "rf-mu-title";
    const idx = this.formatNumber(value?.now_index);
    const args = Array.isArray(value?.args) ? value.args.map(n => this.formatNumber(n)).join(", ") : "";
    headline.textContent = `MuOp index=${idx ?? "?"} args=(${args})`;
    wrapper.appendChild(headline);

    const funcLine = document.createElement("div");
    funcLine.textContent = `Function: ${this.describeFunction(value?.function)}`;
    wrapper.appendChild(funcLine);

    if (value?.process) {
      const subHeading = document.createElement("div");
      subHeading.className = "rf-subheading";
      subHeading.textContent = "Current inner process:";
      wrapper.appendChild(subHeading);
      wrapper.appendChild(this.renderProcessNode(value.process));
    }
    return wrapper;
  }

  describeFunction(node) {
    if (!node) return "(unknown)";
    if (typeof node === "string") return this.prettyName(node);
    const keys = Object.keys(node);
    if (keys.length !== 1) {
      return this.safeStringify(node);
    }
    const tag = keys[0];
    const value = node[tag];
    switch (tag) {
      case "ZeroConstant":
        return "ZERO";
      case "Successor":
        return "SUCC";
      case "Projection":
        return `PROJ[${value?.parameter_length}, ${value?.projection_num}]`;
      case "Composition": {
        const outer = this.describeFunction(value?.outer_func);
        const inner = Array.isArray(value?.inner_funcs)
          ? value.inner_funcs.map(func => this.describeFunction(func)).join(", ")
          : "";
        return `COMP[${outer}: (${inner})]`;
      }
      case "PrimitiveRecursion": {
        const zero = this.describeFunction(value?.zero_func);
        const succ = this.describeFunction(value?.succ_func);
        return `PRIM[z: ${zero}, s: ${succ}]`;
      }
      case "MuOperator":
        return `MU[${this.describeFunction(value?.mu_func)}]`;
      default:
        return `[${tag}]`;
    }
  }

  prettyName(name) {
    switch (name) {
      case "ZeroConstant":
        return "ZERO";
      case "Successor":
        return "SUCC";
      default:
        return name;
    }
  }

  makeLeaf(text) {
    const span = document.createElement("span");
    span.className = "rf-leaf";
    span.textContent = text;
    return span;
  }

  makeTextNode(text) {
    return document.createTextNode(text);
  }

  extractVariant(node) {
    if (!node || typeof node !== "object") {
      if (typeof node === "string") {
        return { tag: node, value: node };
      }
      return null;
    }
    const keys = Object.keys(node);
    if (keys.length !== 1) return null;
    return { tag: keys[0], value: node[keys[0]] };
  }

  formatNumber(raw) {
    if (typeof raw === "number" && Number.isFinite(raw)) {
      return raw;
    }
    if (raw && typeof raw === "object" && "Number" in raw) {
      return this.formatNumber(raw.Number);
    }
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : raw;
  }

  safeStringify(value) {
    try {
      return JSON.stringify(value);
    } catch (_) {
      return String(value);
    }
  }
}
