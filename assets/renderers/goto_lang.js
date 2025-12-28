// assets/renderers/goto_lang.js
// SnapshotRenderer for goto_lang: program listing + environment table.

export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;

    this.stateContainer.replaceChildren();

    this.metaLine = document.createElement("div");
    this.metaLine.className = "goto-meta";

    const codeSection = document.createElement("section");
    codeSection.className = "goto-section goto-code-section";
    const codeHeading = document.createElement("h4");
    codeHeading.textContent = "Program";
    this.codeList = document.createElement("ol");
    this.codeList.className = "goto-code-list";
    codeSection.append(codeHeading, this.codeList);

    const envSection = document.createElement("section");
    envSection.className = "goto-section goto-env-section";
    const envHeading = document.createElement("h4");
    envHeading.textContent = "Environment";
    this.envTable = document.createElement("table");
    this.envTable.className = "goto-env-table";
    const head = this.envTable.createTHead();
    const headRow = head.insertRow();
    ["Variable", "Value"].forEach(text => {
      headRow.insertCell().textContent = text;
    });
    this.envBody = this.envTable.createTBody();
    envSection.append(envHeading, this.envTable);

    this.hintLine = document.createElement("div");
    this.hintLine.className = "goto-hint";
    this.hintLine.textContent = "PC is highlighted; program shows decoded commands.";

    this.stateContainer.append(this.metaLine, codeSection, envSection, this.hintLine);
  }

  draw(state) {
    this.renderMeta(state);
    this.renderCode(state);
    this.renderEnvironment(state);
  }

  renderMeta(state) {
    if (!this.metaLine) return;
    if (!state) {
      this.metaLine.textContent = "state: (none)";
      return;
    }
    const pcText = formatNumber(state.pc, "?");
    const terminated = booleanFrom(state.is_terminated ?? state.terminated ?? null);
    const status = [`pc: ${pcText}`];
    const commands = this.extractCommands(state);
    if (commands.length) {
      status.push(`len: ${commands.length}`);
    }
    if (terminated === true) {
      status.push("(terminated)");
    }
    this.metaLine.textContent = status.join(" | ");
  }

  renderCode(state) {
    if (!this.codeList) return;
    this.codeList.replaceChildren();
    const commands = this.extractCommands(state);
    if (!commands.length) {
      const placeholder = document.createElement("li");
      placeholder.textContent = "(empty program)";
      this.codeList.appendChild(placeholder);
      return;
    }
    const pc = toIndex(state && state.pc, commands.length);

    commands.forEach((command, idx) => {
      const line = document.createElement("li");
      line.className = "goto-code-line";
      line.textContent = this.describeCommand(command);
      if (pc === idx) {
        line.classList.add("goto-code-current");
      }
      this.codeList.appendChild(line);
    });
  }

  renderEnvironment(state) {
    if (!this.envBody) return;
    while (this.envBody.firstChild) {
      this.envBody.removeChild(this.envBody.firstChild);
    }
    const envEntries = state && state.env && Array.isArray(state.env.env) ? state.env.env : [];

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
      row.insertCell().textContent = this.formatVar(rawVar);
      const valueCell = row.insertCell();
      valueCell.textContent = formatNumber(rawValue, "");
    });
  }

  extractCommands(state) {
    if (!state) return [];
    const raw = state.commands;
    if (Array.isArray(raw)) {
      if (raw.length === 1 && Array.isArray(raw[0])) return raw[0];
      return raw;
    }
    if (raw && typeof raw === "object") {
      if (Array.isArray(raw[0])) return raw[0];
      if (Array.isArray(raw["0"])) return raw["0"];
    }
    return [];
  }

  describeCommand(command) {
    const variant = extractVariant(command);
    if (!variant) {
      return safeStringify(command);
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
        const [dst, src] = asTuple(value, 2);
        return `cpy ${this.formatVar(dst)} <- ${this.formatVar(src)}`;
      }
      case "Ifnz": {
        const [varName, target] = asTuple(value, 2);
        return `ifnz ${this.formatVar(varName)} : ${formatNumber(target, "?")}`;
      }
      default:
        return `[${tag}]`;
    }
  }

  formatVar(raw) {
    if (typeof raw === "string") {
      return raw;
    }
    if (Array.isArray(raw) && typeof raw[0] === "string") {
      return raw[0];
    }
    if (raw == null) return "?";
    return String(raw);
  }
}

function extractVariant(node) {
  if (!node || typeof node !== "object") return null;
  const keys = Object.keys(node);
  if (keys.length !== 1) return null;
  return { tag: keys[0], value: node[keys[0]] };
}

function asTuple(value, len) {
  if (Array.isArray(value) && value.length >= len) {
    return value;
  }
  if (len === 1) {
    return [value];
  }
  return Array(len).fill(undefined);
}

function formatNumber(raw, fallback) {
  const big = toBigInt(raw);
  return big == null ? fallback : big.toString(10);
}

function toIndex(raw, maxLen) {
  const big = toBigInt(raw);
  if (big == null || maxLen <= 0) return undefined;
  if (big < 0n || big >= BigInt(maxLen)) return undefined;
  return Number(big);
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

function booleanFrom(raw) {
  if (typeof raw === "boolean") return raw;
  if (raw === 1) return true;
  if (raw === 0) return false;
  return undefined;
}

function safeStringify(value) {
  try {
    return JSON.stringify(value);
  } catch (_) {
    return String(value);
  }
}
