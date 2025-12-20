// assets/renderers/tiny_isa.js
// SnapshotRenderer for tiny_isa: registers, code listing, and memory data.

export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
    this.stateContainer.replaceChildren();

    this.metaLine = document.createElement("div");
    this.metaLine.className = "tiny-isa-meta";

    const codeSection = document.createElement("section");
    codeSection.className = "tiny-isa-section tiny-isa-code";
    const codeHeading = document.createElement("h4");
    codeHeading.textContent = "Program";
    this.codeList = document.createElement("ol");
    this.codeList.className = "tiny-isa-code-list";
    codeSection.append(codeHeading, this.codeList);

    const regSection = document.createElement("section");
    regSection.className = "tiny-isa-section tiny-isa-registers";
    const regHeading = document.createElement("h4");
    regHeading.textContent = "Registers";
    this.regTable = document.createElement("table");
    this.regTable.className = "tiny-isa-reg-table";
    const regHead = this.regTable.createTHead();
    const regHeadRow = regHead.insertRow();
    ["Register", "Value (dec)", "Value (hex)"].forEach(text => {
      regHeadRow.insertCell().textContent = text;
    });
    this.regBody = this.regTable.createTBody();
    regSection.append(regHeading, this.regTable);

    const dataSection = document.createElement("section");
    dataSection.className = "tiny-isa-section tiny-isa-data";
    const dataHeading = document.createElement("h4");
    dataHeading.textContent = "Memory (data)";
    this.dataTable = document.createElement("table");
    this.dataTable.className = "tiny-isa-data-table";
    const dataHead = this.dataTable.createTHead();
    const dataHeadRow = dataHead.insertRow();
    ["Addr", "Value (dec)", "Value (hex)"].forEach(text => {
      dataHeadRow.insertCell().textContent = text;
    });
    this.dataBody = this.dataTable.createTBody();
    dataSection.append(dataHeading, this.dataTable);

    this.stateContainer.append(
      this.metaLine,
      codeSection,
      regSection,
      dataSection,
    );
  }

  draw(state) {
    this.renderMeta(state);
    this.renderCode(state);
    this.renderRegisters(state);
    this.renderData(state);
  }

  renderMeta(state) {
    if (!this.metaLine) return;
    if (!state) {
      this.metaLine.textContent = "state: (not initialized)";
      return;
    }
    const memory = Array.isArray(state.memory) ? state.memory : [];
    const codeLenIdx = this.toSafeIndex(state.code_len, memory.length);
    const pcIdx = this.toSafeIndex(state.pc, memory.length);
    const pcText = this.formatNumber(state.pc) ?? "?";
    const codeLenText = this.formatNumber(state.code_len) ?? "?";
    const parts = [`pc: ${pcText}`, `code_len: ${codeLenText}`, `mem: ${memory.length}`];
    if (pcIdx >= memory.length && memory.length > 0) {
      parts.push("(pc out of bounds)");
    }
    this.metaLine.textContent = parts.join(" | ");
  }

  renderCode(state) {
    if (!this.codeList) return;
    this.codeList.replaceChildren();
    if (!state) {
      const li = document.createElement("li");
      li.textContent = "(no state)";
      this.codeList.appendChild(li);
      return;
    }
    const memory = Array.isArray(state.memory) ? state.memory : [];
    const codeLen = this.toSafeIndex(state.code_len, memory.length);
    if (!memory.length || codeLen === 0) {
      const li = document.createElement("li");
      li.textContent = "(empty program)";
      this.codeList.appendChild(li);
      return;
    }
    const pcIdx = this.toSafeIndex(state.pc, memory.length);
    memory.slice(0, codeLen).forEach((cell, idx) => {
      const li = document.createElement("li");
      li.className = "tiny-isa-code-line";
      const bytes = this.toBytes(cell);
      const opcode = this.decodeInstruction(bytes);
      li.textContent = `${idx}: ${opcode}`;
      if (idx === pcIdx) {
        li.classList.add("tiny-isa-code-current");
      }
      this.codeList.appendChild(li);
    });
  }

  renderRegisters(state) {
    if (!this.regBody) return;
    while (this.regBody.firstChild) {
      this.regBody.removeChild(this.regBody.firstChild);
    }
    if (!state) {
      const row = this.regBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 3;
      cell.textContent = "(no registers)";
      cell.style.textAlign = "center";
      return;
    }
    const regs = Array.isArray(state.registers) ? state.registers : [];
    for (let i = 0; i < 4; i += 1) {
      const row = this.regBody.insertRow();
      row.insertCell().textContent = `R${i}`;
      const value = regs[i];
      row.insertCell().textContent = this.formatNumber(value) ?? "0";
      const bytes = this.toBytes(value);
      row.insertCell().textContent = bytes ? this.formatHex(bytes) : "0x00";
    }
  }

  renderData(state) {
    if (!this.dataBody) return;
    while (this.dataBody.firstChild) {
      this.dataBody.removeChild(this.dataBody.firstChild);
    }
    if (!state) {
      const row = this.dataBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 3;
      cell.textContent = "(no memory)";
      cell.style.textAlign = "center";
      return;
    }
    const memory = Array.isArray(state.memory) ? state.memory : [];
    const codeLen = this.toSafeIndex(state.code_len, memory.length);
    const data = memory.slice(codeLen);
    if (!data.length) {
      const row = this.dataBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 3;
      cell.textContent = "(empty)";
      cell.style.textAlign = "center";
      return;
    }
    data.forEach((cell, idx) => {
      const row = this.dataBody.insertRow();
      row.insertCell().textContent = String(codeLen + idx);
      row.insertCell().textContent = this.formatNumber(cell) ?? "0";
      const bytes = this.toBytes(cell);
      row.insertCell().textContent = bytes ? this.formatHex(bytes) : "0x00";
    });
  }

  decodeInstruction(bytes) {
    if (!Array.isArray(bytes) || bytes.length === 0) {
      return "HALT";
    }
    const op = bytes[0] ?? 0;
    const opcode = (op & 0xf0) >> 4;
    const reg1 = (op & 0x0c) >> 2;
    const reg2 = op & 0x03;
    const imm = bytes.slice(1);
    const immText = this.formatNumber(imm) ?? "0";

    switch (opcode) {
      case 0x0:
        return "HALT";
      case 0x1:
        return "NOP";
      case 0x2:
        return `LOADI ${this.regName(reg1)}, ${immText}`;
      case 0x3:
        return `LOAD ${this.regName(reg1)}, [${immText}]`;
      case 0x4:
        return `STORE ${this.regName(reg1)}, [${immText}]`;
      case 0x5:
        return `MOV ${this.regName(reg1)}, ${this.regName(reg2)}`;
      case 0x6:
        return `ADD ${this.regName(reg1)}, ${this.regName(reg2)}`;
      case 0x7:
        return `SUB ${this.regName(reg1)}, ${this.regName(reg2)}`;
      case 0x8:
        return `JMP ${this.regName(reg1)}`;
      case 0x9:
        return `JMP ${immText}`;
      case 0xa:
        return `JLT ${this.regName(reg1)}, ${this.regName(reg2)}, ${immText}`;
      default: {
        const hex = this.formatHex(bytes);
        return `DB ${hex}`;
      }
    }
  }

  regName(code) {
    if (code === 0) return "R0";
    if (code === 1) return "R1";
    if (code === 2) return "R2";
    if (code === 3) return "R3";
    return `R?(${code})`;
  }

  toSafeIndex(raw, maxLen) {
    const num = this.formatNumber(raw);
    if (num == null) return 0;
    const asBigInt = this.toBigInt(this.toBytes(raw));
    if (asBigInt == null) {
      const parsed = Number(num);
      if (Number.isFinite(parsed)) {
        return Math.max(0, Math.min(maxLen, Math.floor(parsed)));
      }
      return 0;
    }
    if (asBigInt > BigInt(maxLen)) return maxLen;
    if (asBigInt < 0n) return 0;
    return Number(asBigInt);
  }

  formatNumber(raw) {
    if (raw == null) return null;
    if (typeof raw === "number" && Number.isFinite(raw)) {
      return String(raw);
    }
    if (typeof raw === "bigint") {
      return raw.toString(10);
    }
    if (typeof raw === "string") {
      return raw;
    }
    const bytes = this.toBytes(raw);
    if (bytes) {
      const value = this.toBigInt(bytes);
      return value == null ? null : value.toString(10);
    }
    return null;
  }

  toBytes(raw) {
    if (Array.isArray(raw)) {
      return raw;
    }
    if (raw && typeof raw === "object") {
      if (Array.isArray(raw.Number)) return raw.Number;
      if ("Number" in raw) return this.toBytes(raw.Number);
      if (Array.isArray(raw["0"])) return raw["0"];
      if (Array.isArray(raw.value)) return raw.value;
    }
    return null;
  }

  toBigInt(bytes) {
    if (!Array.isArray(bytes)) return null;
    let acc = 0n;
    for (let i = bytes.length - 1; i >= 0; i -= 1) {
      const b = bytes[i] ?? 0;
      acc = (acc << 8n) + BigInt(b);
    }
    return acc;
  }

  formatHex(bytes) {
    if (!Array.isArray(bytes) || bytes.length === 0) {
      return "0x00";
    }
    const parts = bytes
      .slice()
      .reverse()
      .map(b => {
        const num = typeof b === "number" ? b : Number(b) || 0;
        return num.toString(16).padStart(2, "0").toUpperCase();
      });
    return `0x${parts.join("")}`;
  }
}
