// assets/renderers/goto_lang.js
// Renderer for goto_lang that shows commands with the current PC and an Environment table.

export class Renderer {
  constructor(vm, stateContainer, outputContainer) {
    this.vm = vm;
    this.stateContainer = stateContainer;
    this.outputContainer = outputContainer;

    this.stateContainer.replaceChildren();
    this.outputContainer.replaceChildren();

    this.metaLine = document.createElement("div");
    this.metaLine.className = "goto-meta";

    this.codeWrapper = document.createElement("div");
    this.codeWrapper.className = "goto-code-wrapper";
    const codeLabel = document.createElement("div");
    codeLabel.textContent = "Program";

    this.codePre = document.createElement("pre");
    this.codePre.className = "goto-code";
    this.codeElement = document.createElement("code");
    this.codePre.appendChild(this.codeElement);
    this.codeWrapper.append(codeLabel, this.codePre);

    const envLabel = document.createElement("div");
    envLabel.textContent = "Environment";
    this.envTable = document.createElement("table");
    this.envTable.className = "goto-env-table";
    const head = this.envTable.createTHead();
    const headRow = head.insertRow();
    ["Variable", "Value"].forEach(text => {
      headRow.insertCell().textContent = text;
    });
    this.envBody = this.envTable.createTBody();

    this.stateContainer.append(this.metaLine, this.codeWrapper, envLabel, this.envTable);

    this.outputMessage = document.createElement("div");
    this.outputMessage.className = "goto-output";
    this.outputContainer.appendChild(this.outputMessage);
  }

  drawState(state) {
    this.renderMeta(state);
    this.renderCode(state);
    this.renderEnvironment(state);
  }

  drawOutput(output) {
    if (!this.outputMessage) return;
    if (output === undefined) {
      this.outputMessage.textContent = "";
      return;
    }
    if (output === true) {
      this.outputMessage.textContent = "Terminated.";
    } else if (output === false) {
      this.outputMessage.textContent = "Running...";
    } else {
      this.outputMessage.textContent = `output: ${this.safeStringify(output)}`;
    }
  }

  renderMeta(state) {
    if (!this.metaLine) return;
    if (!state) {
      this.metaLine.textContent = "state: (none)";
      return;
    }
    const pc = this.coerceNumber(state.pc);
    const terminated = this.booleanFrom(state.is_terminated ?? (state.terminated ?? null));
    const status = [];
    status.push(`pc: ${pc ?? "?"}`);
    if (typeof state.commands === "object" && Array.isArray(state.commands)) {
      status.push(`len: ${state.commands.length}`);
    }
    if (terminated === true) {
      status.push("(terminated)");
    }
    this.metaLine.textContent = status.join(" | ");
  }

  renderCode(state) {
    if (!this.codeElement) return;
    this.codeElement.replaceChildren();
    const commands = state && Array.isArray(state.commands) ? state.commands : [];
    if (!commands.length) {
      this.codeElement.textContent = "(empty program)";
      return;
    }
    const pc = this.coerceNumber(state && state.pc);

    commands.forEach((command, idx) => {
      const line = document.createElement("div");
      line.className = "goto-code-line";
      line.textContent = `${idx}: ${this.describeCommand(command)}`;
      if (pc === idx) {
        line.style.color = "red";
        line.style.fontWeight = "bold";
      }
      this.codeElement.appendChild(line);
    });
  }

  renderEnvironment(state) {
    if (!this.envBody) return;
    while (this.envBody.firstChild) {
      this.envBody.removeChild(this.envBody.firstChild);
    }
    const envEntries =
      state && state.env && Array.isArray(state.env.env) ? state.env.env : [];

    if (!envEntries.length) {
      const row = this.envBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 2;
      cell.textContent = "(empty)";
      cell.style.textAlign = "center";
      return;
    }

    envEntries.forEach(entry => {
      const [rawVar, rawValue] = Array.isArray(entry) ? entry : [null, null];
      const row = this.envBody.insertRow();
      row.insertCell().textContent = rawVar ?? "";
      const valueCell = row.insertCell();
      const value = this.coerceNumber(rawValue);
      valueCell.textContent = value ?? "";
    });
  }

  describeCommand(command) {
    const variant = this.extractVariant(command);
    if (!variant) {
      return this.safeStringify(command);
    }
    const { tag, value } = variant;
    switch (tag) {
      case "Clr":
        return `clr ${this.formatVar(value)}`;
      case "Inc":
        return `inc ${this.formatVar(value)}`;
      case "Dec":
        return `dec ${this.formatVar(value)}`;
      case "Cpy": {
        const [dst, src] = this.asTuple(value, 2);
        return `cpy ${this.formatVar(dst)} <- ${this.formatVar(src)}`;
      }
      case "Ifz": {
        const [varName, target] = this.asTuple(value, 2);
        return `ifz ${this.formatVar(varName)} : ${this.coerceNumber(target) ?? "?"}`;
      }
      default:
        return `[${tag}]`;
    }
  }

  extractVariant(node) {
    if (!node || typeof node !== "object") return null;
    const keys = Object.keys(node);
    if (keys.length !== 1) return null;
    return { tag: keys[0], value: node[keys[0]] };
  }

  asTuple(value, len) {
    if (Array.isArray(value) && value.length >= len) {
      return value;
    }
    if (len === 1) {
      return [value];
    }
    return Array(len).fill(undefined);
  }

  formatVar(raw) {
    if (typeof raw === "string") {
      return raw;
    }
    if (raw == null) return "?";
    return String(raw);
  }

  coerceNumber(raw) {
    if (typeof raw === "number" && Number.isFinite(raw)) {
      return raw;
    }
    if (Array.isArray(raw) && raw.length === 1) {
      return this.coerceNumber(raw[0]);
    }
    if (raw && typeof raw === "object") {
      if ("Number" in raw) {
        return this.coerceNumber(raw.Number);
      }
      if ("pc" in raw) {
        return this.coerceNumber(raw.pc);
      }
    }
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : undefined;
  }

  booleanFrom(raw) {
    if (typeof raw === "boolean") return raw;
    if (raw === 1) return true;
    if (raw === 0) return false;
    return undefined;
  }

  safeStringify(value) {
    try {
      return JSON.stringify(value);
    } catch (_) {
      return String(value);
    }
  }
}
