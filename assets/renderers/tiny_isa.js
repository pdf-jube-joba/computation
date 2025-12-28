// assets/renderers/tiny_isa.js
// SnapshotRenderer for tiny_isa: registers + unified memory listing.

export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
    this.stateContainer.replaceChildren();

    this.metaLine = document.createElement("div");
    this.metaLine.className = "tiny-isa-meta";

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

    const memSection = document.createElement("section");
    memSection.className = "tiny-isa-section tiny-isa-memory";
    const memHeading = document.createElement("h4");
    memHeading.textContent = "Memory";
    this.memTable = document.createElement("table");
    this.memTable.className = "tiny-isa-memory-table";
    const memHead = this.memTable.createTHead();
    const memHeadRow = memHead.insertRow();
    ["Addr", "Value (dec)", "Value (hex)", "Decoded"].forEach(text => {
      memHeadRow.insertCell().textContent = text;
    });
    this.memBody = this.memTable.createTBody();
    memSection.append(memHeading, this.memTable);

    this.stateContainer.append(this.metaLine, regSection, memSection);
  }

  draw(state) {
    this.renderMeta(state);
    this.renderRegisters(state);
    this.renderMemory(state);
  }

  renderMeta(state) {
    if (!this.metaLine) return;
    if (!state) {
      this.metaLine.textContent = "state: (not initialized)";
      return;
    }
    const memory = Array.isArray(state.memory) ? state.memory : [];
    const pcText = formatNumber(state.pc) ?? "?";
    const codeLenText = formatNumber(state.code_len) ?? "?";
    const parts = [`pc: ${pcText}`, `code_len: ${codeLenText}`, `mem: ${memory.length}`];
    this.metaLine.textContent = parts.join(" | ");
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
      row.insertCell().textContent = formatNumber(value) ?? "0";
      const bytes = toBytes(value);
      row.insertCell().textContent = bytes ? formatHex(bytes) : "0x00";
    }
  }

  renderMemory(state) {
    if (!this.memBody) return;
    while (this.memBody.firstChild) {
      this.memBody.removeChild(this.memBody.firstChild);
    }
    if (!state) {
      const row = this.memBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 4;
      cell.textContent = "(no memory)";
      cell.style.textAlign = "center";
      return;
    }
    const memory = Array.isArray(state.memory) ? state.memory : [];
    if (!memory.length) {
      const row = this.memBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 4;
      cell.textContent = "(empty)";
      cell.style.textAlign = "center";
      return;
    }

    const codeLenIdx = toSafeIndex(state.code_len, memory.length);
    const pcIdx = toSafeIndex(state.pc, memory.length);

    memory.forEach((cell, idx) => {
      if (idx === codeLenIdx) {
        const marker = this.memBody.insertRow();
        marker.className = "tiny-isa-code-boundary";
        const markerCell = marker.insertCell();
        markerCell.colSpan = 4;
        markerCell.textContent = "-- code_len boundary --";
        markerCell.style.textAlign = "center";
      }

      const row = this.memBody.insertRow();
      row.insertCell().textContent = String(idx);
      row.insertCell().textContent = formatNumber(cell) ?? "0";
      const bytes = toBytes(cell);
      row.insertCell().textContent = bytes ? formatHex(bytes) : "0x00";
      row.insertCell().textContent = decodeInstruction(bytes);

      if (idx === pcIdx) {
        row.classList.add("tiny-isa-code-current");
      }
    });

    if (codeLenIdx === memory.length) {
      const marker = this.memBody.insertRow();
      marker.className = "tiny-isa-code-boundary";
      const markerCell = marker.insertCell();
      markerCell.colSpan = 4;
      markerCell.textContent = "-- code_len boundary --";
      markerCell.style.textAlign = "center";
    }
  }
}

function decodeInstruction(bytes) {
  if (!Array.isArray(bytes) || bytes.length === 0) {
    return "NOP";
  }
  const op = bytes[0] ?? 0;
  const opcode = (op & 0xf0) >> 4;
  const reg1 = (op & 0x0c) >> 2;
  const reg2 = op & 0x03;
  const immBytes = bytes.slice(1);
  const immText = formatNumber(bytesToBigInt(immBytes)) ?? "0";

  switch (opcode) {
    case 0x0:
      return "NOP";
    case 0x1:
      return "HALT";
    case 0x2:
      return `LOADI ${regName(reg1)}, ${immText}`;
    case 0x3:
      return `LOAD ${regName(reg1)}, [${immText}]`;
    case 0x4:
      return `STORE ${regName(reg1)}, [${immText}]`;
    case 0x5:
      return `MOV ${regName(reg1)}, ${regName(reg2)}`;
    case 0x6:
      return `ADD ${regName(reg1)}, ${regName(reg2)}`;
    case 0x7:
      return `SUB ${regName(reg1)}, ${regName(reg2)}`;
    case 0x8:
      return `READPC ${regName(reg1)}`;
    case 0x9:
      return `JMP ${regName(reg1)}`;
    case 0xa:
      return `JMP ${immText}`;
    case 0xb:
      return `JMPREL ${regName(reg1)}`;
    case 0xc:
      return `JMPREL ${immText}`;
    case 0xd:
      return `JLTREL ${regName(reg1)}, ${regName(reg2)}, ${immText}`;
    case 0xe:
      return `JLTREL_BACK ${regName(reg1)}, ${regName(reg2)}, ${immText}`;
    default: {
      const hex = formatHex(bytes);
      return `DB ${hex}`;
    }
  }
}

function regName(code) {
  if (code === 0) return "R0";
  if (code === 1) return "R1";
  if (code === 2) return "R2";
  if (code === 3) return "R3";
  return `R?(${code})`;
}

function toSafeIndex(raw, maxLen) {
  const big = toBigInt(raw);
  if (big == null) return 0;
  if (big < 0n) return 0;
  const max = BigInt(maxLen);
  if (big > max) return maxLen;
  return Number(big);
}

function formatNumber(raw) {
  if (raw == null) return null;
  const big = toBigInt(raw);
  return big == null ? null : big.toString(10);
}

function toBytes(raw) {
  if (Array.isArray(raw) && raw.every(isByte)) {
    return raw;
  }
  if (raw && typeof raw === "object" && "Number" in raw) {
    return toBytes(raw.Number);
  }
  const big = toBigInt(raw);
  if (big == null) return null;
  return bigIntToBytes(big);
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
  return null;
}

function bigIntToBytes(value) {
  if (value === 0n) return [0];
  const bytes = [];
  let cur = value;
  while (cur > 0n) {
    bytes.push(Number(cur & 0xffn));
    cur >>= 8n;
  }
  return bytes;
}

function bytesToBigInt(bytes) {
  if (!Array.isArray(bytes) || bytes.length === 0) return 0n;
  let acc = 0n;
  for (let i = bytes.length - 1; i >= 0; i -= 1) {
    const b = bytes[i] ?? 0;
    acc = (acc << 8n) + BigInt(b);
  }
  return acc;
}

function formatHex(bytes) {
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

function isByte(value) {
  return Number.isInteger(value) && value >= 0 && value <= 255;
}
