// assets/renderers/recursive_function.js
// SnapshotRenderer for recursive_function machines: shows current Process tree.

export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;

    this.stateContainer.replaceChildren();

    this.statusLine = document.createElement("div");
    this.statusLine.className = "rf-status";

    const processSection = document.createElement("section");
    processSection.className = "rf-section";
    const processHeading = document.createElement("h4");
    processHeading.textContent = "Current Process";
    this.processContainer = document.createElement("div");
    this.processContainer.className = "rf-process";
    processSection.append(processHeading, this.processContainer);

    this.stateContainer.append(this.statusLine, processSection);
  }

  draw(state) {
    this.renderStatus(state);
    this.renderProcess(state);
  }

  renderStatus(state) {
    if (!this.statusLine) return;
    if (!state) {
      this.statusLine.textContent = "state: (not initialized)";
      return;
    }
    const functionText = describeFunction(state.function) ?? "(unknown)";
    const inputText = Array.isArray(state.input)
      ? state.input.map(n => formatNumber(n, "?")).join(", ")
      : "";
    const process = state.process ?? state;
    const variant = extractVariant(process);
    const parts = [`function: ${functionText}`, `input: (${inputText})`];
    if (variant?.tag === "Result") {
      parts.push(`result: ${formatNumber(variant.value, "?")}`);
    } else if (variant?.tag) {
      parts.push(`status: evaluating (${variant.tag})`);
    }
    this.statusLine.textContent = parts.join(" | ");
  }

  renderProcess(state) {
    if (!this.processContainer) return;
    this.processContainer.replaceChildren();
    if (!state) {
      this.processContainer.textContent = "(no process)";
      return;
    }
    const process = state.process ?? state;
    this.processContainer.appendChild(this.renderProcessNode(process));
  }

  renderProcessNode(node) {
    const variant = extractVariant(node);
    if (!variant) {
      return this.makeTextNode(safeStringify(node ?? "(unknown)"));
    }
    const { tag, value } = variant;
    switch (tag) {
      case "Result":
        return this.makeLeaf(`Result: ${formatNumber(value, "?")}`);
      case "Comp":
        return this.renderCompNode(value);
      case "MuOpComp":
        return this.renderMuNode(value);
      default:
        return this.makeLeaf(`[${tag}] ${safeStringify(value)}`);
    }
  }

  renderCompNode(value) {
    const wrapper = document.createElement("div");
    wrapper.className = "rf-comp";
    const title = document.createElement("div");
    title.className = "rf-comp-title";
    title.textContent = `Call ${describeFunction(value?.function)} (${(value?.args || []).length} args)`;
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
    const idx = formatNumber(value?.now_index, "?");
    const args = Array.isArray(value?.args)
      ? value.args.map(n => formatNumber(n, "?")).join(", ")
      : "";
    headline.textContent = `MuOp index=${idx} args=(${args})`;
    wrapper.appendChild(headline);

    const funcLine = document.createElement("div");
    funcLine.textContent = `Function: ${describeFunction(value?.function)}`;
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

  makeLeaf(text) {
    const span = document.createElement("span");
    span.className = "rf-leaf";
    span.textContent = text;
    return span;
  }

  makeTextNode(text) {
    return document.createTextNode(text);
  }
}

function extractVariant(node) {
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

function describeFunction(node) {
  if (!node) return "(unknown)";
  if (typeof node === "string") return prettyName(node);
  const keys = Object.keys(node);
  if (keys.length !== 1) {
    return safeStringify(node);
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
      const outer = describeFunction(value?.outer_func);
      const inner = Array.isArray(value?.inner_funcs)
        ? value.inner_funcs.map(func => describeFunction(func)).join(", ")
        : "";
      return `COMP[${outer}: (${inner})]`;
    }
    case "PrimitiveRecursion": {
      const zero = describeFunction(value?.zero_func);
      const succ = describeFunction(value?.succ_func);
      return `PRIM[z: ${zero}, s: ${succ}]`;
    }
    case "MuOperator":
      return `MU[${describeFunction(value?.mu_func)}]`;
    default:
      return `[${tag}]`;
  }
}

function prettyName(name) {
  switch (name) {
    case "ZeroConstant":
      return "ZERO";
    case "Successor":
      return "SUCC";
    default:
      return name;
  }
}

function formatNumber(raw, fallback) {
  const big = toBigInt(raw);
  return big == null ? fallback : big.toString(10);
}

function toBigInt(raw) {
  if (typeof raw === "bigint") return raw;
  if (typeof raw === "number" && Number.isFinite(raw)) {
    return BigInt(Math.trunc(raw));
  }
  if (typeof raw === "string" && raw.trim() !== "") {
    try {
      return BigInt(raw);
    } catch (_) {
      return null;
    }
  }
  if (raw && typeof raw === "object" && "Number" in raw) {
    return toBigInt(raw.Number);
  }
  return null;
}

function safeStringify(value) {
  try {
    return JSON.stringify(value);
  } catch (_) {
    return String(value);
  }
}
