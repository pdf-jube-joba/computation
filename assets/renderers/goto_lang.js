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
    const pc = this.coerceNumber(state.pc);
    const pcText = this.formatNumber(state.pc);
    const terminated = this.booleanFrom(state.is_terminated ?? (state.terminated ?? null));
    const status = [];
    status.push(`pc: ${pcText}`);
    if (typeof state.commands === "object" && Array.isArray(state.commands)) {
      status.push(`len: ${state.commands.length}`);
    }
    if (terminated === true) {
      status.push("(terminated)");
    }
    this.metaLine.textContent = status.join(" | ");
  }

  renderCode(state) {
    if (!this.codeList) return;
    this.codeList.replaceChildren();
    const commands = state && Array.isArray(state.commands) ? state.commands : [];
    if (!commands.length) {
      const placeholder = document.createElement("li");
      placeholder.textContent = "(empty program)";
      this.codeList.appendChild(placeholder);
      return;
    }
    const pc = this.coerceNumber(state && state.pc);

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
      valueCell.textContent = this.formatNumber(rawValue, "");
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
      case "Ifnz": {
        const [varName, target] = this.asTuple(value, 2);
        return `ifnz ${this.formatVar(varName)} : ${this.formatNumber(target)}`;
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
    if (typeof raw === "bigint") {
      return raw <= BigInt(Number.MAX_SAFE_INTEGER) ? Number(raw) : undefined;
    }
    const bytes = this.extractBytes(raw);
    if (bytes) {
      const value = this.bytesToBigInt(bytes);
      if (value <= BigInt(Number.MAX_SAFE_INTEGER)) {
        return Number(value);
      }
      return undefined;
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

  formatNumber(raw, fallback = "?") {
    const asNumber = this.coerceNumber(raw);
    if (asNumber !== undefined) {
      return String(asNumber);
    }
    const asBig = this.coerceBigInt(raw);
    if (asBig !== undefined) {
      return asBig.toString();
    }
    return fallback;
  }

  coerceBigInt(raw) {
    if (typeof raw === "bigint") return raw;
    const bytes = this.extractBytes(raw);
    if (bytes) {
      return this.bytesToBigInt(bytes);
    }
    return undefined;
  }

  extractBytes(raw) {
    if (raw instanceof Uint8Array) {
      return Array.from(raw);
    }
    if (raw instanceof ArrayBuffer) {
      return Array.from(new Uint8Array(raw));
    }
    if (Array.isArray(raw) && raw.every(this.isByte)) {
      return raw;
    }
    if (raw && typeof raw === "object" && raw.type === "Buffer" && Array.isArray(raw.data)) {
      return raw.data;
    }
    return undefined;
  }

  isByte(value) {
    return Number.isInteger(value) && value >= 0 && value <= 255;
  }

  bytesToBigInt(bytes) {
    let value = 0n;
    for (let i = bytes.length - 1; i >= 0; i -= 1) {
      value = (value << 8n) + BigInt(bytes[i]);
    }
    return value;
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
