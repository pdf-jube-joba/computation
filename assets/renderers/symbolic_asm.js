// assets/renderers/symbolic_asm.js
// SnapshotRenderer for symbolic_asm: registers + code listing + data table.

export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
    this.stateContainer.replaceChildren();

    this.metaLine = document.createElement("div");
    this.metaLine.className = "symbolic-asm-meta";

    const regSection = document.createElement("section");
    regSection.className = "symbolic-asm-section symbolic-asm-registers";
    const regHeading = document.createElement("h4");
    regHeading.textContent = "Registers";
    this.regTable = document.createElement("table");
    this.regTable.className = "symbolic-asm-reg-table";
    const regHead = this.regTable.createTHead();
    const regHeadRow = regHead.insertRow();
    ["Register", "Value (dec)", "Value (hex)"].forEach(text => {
      regHeadRow.insertCell().textContent = text;
    });
    this.regBody = this.regTable.createTBody();
    regSection.append(regHeading, this.regTable);

    const codeSection = document.createElement("section");
    codeSection.className = "symbolic-asm-section symbolic-asm-code";
    const codeHeading = document.createElement("h4");
    codeHeading.textContent = "Code";
    this.codeTable = document.createElement("table");
    this.codeTable.className = "symbolic-asm-code-table";
    const codeHead = this.codeTable.createTHead();
    const codeHeadRow = codeHead.insertRow();
    ["Addr", "Label", "Instruction"].forEach(text => {
      codeHeadRow.insertCell().textContent = text;
    });
    this.codeBody = this.codeTable.createTBody();
    codeSection.append(codeHeading, this.codeTable);

    const dataSection = document.createElement("section");
    dataSection.className = "symbolic-asm-section symbolic-asm-data";
    const dataHeading = document.createElement("h4");
    dataHeading.textContent = "Data";
    this.dataTable = document.createElement("table");
    this.dataTable.className = "symbolic-asm-data-table";
    const dataHead = this.dataTable.createTHead();
    const dataHeadRow = dataHead.insertRow();
    ["Addr", "Label", "Value (dec)"].forEach(text => {
      dataHeadRow.insertCell().textContent = text;
    });
    this.dataBody = this.dataTable.createTBody();
    dataSection.append(dataHeading, this.dataTable);

    this.stateContainer.append(this.metaLine, regSection, codeSection, dataSection);
  }

  draw(state) {
    this.renderMeta(state);
    this.renderRegisters(state);
    this.renderCode(state);
    this.renderData(state);
  }

  renderMeta(state) {
    if (!this.metaLine) return;
    if (!state) {
      this.metaLine.textContent = "state: (not initialized)";
      return;
    }
    const codeBlocks = unwrapTupleStruct(state.code);
    const codeLen = Array.isArray(codeBlocks)
      ? codeBlocks.reduce((acc, entry) => {
          const [, instrs] = asTuple(entry, 2);
          return acc + (Array.isArray(instrs) ? instrs.length : 0);
        }, 0)
      : 0;
    const namedData = unwrapTupleStruct(state.named_data);
    const data = Array.isArray(state.data) ? state.data : [];
    const dataLen = (Array.isArray(namedData) ? namedData.length : 0) + data.length;
    const pcText = formatNumber(state.pc) ?? "?";
    this.metaLine.textContent = `pc: ${pcText} | code: ${codeLen} | data: ${dataLen}`;
  }

  renderRegisters(state) {
    if (!this.regBody) return;
    this.regBody.replaceChildren();
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
      row.insertCell().textContent = `r${i}`;
      const value = regs[i];
      row.insertCell().textContent = formatNumber(value) ?? "0";
      const bytes = toBytes(value);
      row.insertCell().textContent = bytes ? formatHex(bytes) : "0x00";
    }
  }

  renderCode(state) {
    if (!this.codeBody) return;
    this.codeBody.replaceChildren();
    if (!state) {
      const row = this.codeBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 3;
      cell.textContent = "(no code)";
      cell.style.textAlign = "center";
      return;
    }
    const codeBlocks = unwrapTupleStruct(state.code);
    if (!Array.isArray(codeBlocks) || codeBlocks.length === 0) {
      const row = this.codeBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 3;
      cell.textContent = "(empty)";
      cell.style.textAlign = "center";
      return;
    }
    const pc = toIndex(state.pc, Number.POSITIVE_INFINITY);
    let addr = 0;
    codeBlocks.forEach(entry => {
      const [labelRaw, instrs] = asTuple(entry, 2);
      const label = formatLabel(labelRaw);
      const labelRow = this.codeBody.insertRow();
      labelRow.className = "symbolic-asm-label";
      const labelCell = labelRow.insertCell();
      labelCell.colSpan = 3;
      labelCell.textContent = label ? label : "(label)";
      labelCell.style.fontWeight = "bold";

      if (!Array.isArray(instrs) || instrs.length === 0) {
        const emptyRow = this.codeBody.insertRow();
        const emptyAddr = emptyRow.insertCell();
        emptyAddr.textContent = String(addr);
        emptyRow.insertCell().textContent = "";
        emptyRow.insertCell().textContent = "(no instruction)";
        if (pc === addr) {
          emptyRow.classList.add("symbolic-asm-code-current");
        }
        addr += 1;
        return;
      }

      instrs.forEach(instr => {
        const row = this.codeBody.insertRow();
        row.insertCell().textContent = String(addr);
        row.insertCell().textContent = "";
        row.insertCell().textContent = describeInstruction(instr);
        if (pc === addr) {
          row.classList.add("symbolic-asm-code-current");
        }
        addr += 1;
      });
    });
  }

  renderData(state) {
    if (!this.dataBody) return;
    this.dataBody.replaceChildren();
    if (!state) {
      const row = this.dataBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 3;
      cell.textContent = "(no data)";
      cell.style.textAlign = "center";
      return;
    }

    const namedData = unwrapTupleStruct(state.named_data);
    const data = Array.isArray(state.data) ? state.data : [];
    const totalLen = (Array.isArray(namedData) ? namedData.length : 0) + data.length;

    if (!totalLen) {
      const row = this.dataBody.insertRow();
      const cell = row.insertCell();
      cell.colSpan = 3;
      cell.textContent = "(empty)";
      cell.style.textAlign = "center";
      return;
    }

    let addr = 0;
    if (Array.isArray(namedData)) {
      namedData.forEach(entry => {
        const [labelRaw, value] = asTuple(entry, 2);
        const row = this.dataBody.insertRow();
        row.insertCell().textContent = String(addr);
        row.insertCell().textContent = formatLabel(labelRaw);
        row.insertCell().textContent = formatNumber(value) ?? "0";
        addr += 1;
      });
    }
    data.forEach(value => {
      const row = this.dataBody.insertRow();
      row.insertCell().textContent = String(addr);
      row.insertCell().textContent = "";
      row.insertCell().textContent = formatNumber(value) ?? "0";
      addr += 1;
    });
  }
}

function describeInstruction(instr) {
  const variant = extractVariant(instr);
  if (!variant) {
    return safeStringify(instr ?? "");
  }
  const { tag, value } = variant;
  switch (tag) {
    case "Nop":
      return "NOP";
    case "Halt":
      return "HLT";
    case "LoadImm":
      return `LDI ${formatRegister(value?.dest)}, ${formatNumber(value?.value) ?? "0"}`;
    case "Load":
      return `LD ${formatRegister(value?.dest)}, ${formatNumber(value?.addr) ?? "0"}`;
    case "LoadLabel":
      return `LDL ${formatRegister(value?.dest)}, ${formatLabel(value?.label)}`;
    case "Store":
      return `ST ${formatRegister(value?.src)}, ${formatNumber(value?.value) ?? "0"}`;
    case "StoreLabel":
      return `STL ${formatRegister(value?.src)}, ${formatLabel(value?.label)}`;
    case "Mov":
      return `MOV ${formatRegister(value?.dest)}, ${formatRegister(value?.src)}`;
    case "Add":
      return `ADD ${formatRegister(value?.dest)}, ${formatRegister(value?.src)}`;
    case "Sub":
      return `SUB ${formatRegister(value?.dest)}, ${formatRegister(value?.src)}`;
    case "ReadPc":
      return `RPC ${formatRegister(value?.dest)}`;
    case "JmpReg":
      return `JMPR ${formatRegister(value?.target)}`;
    case "JmpImm":
      return `JMP ${formatNumber(value?.value) ?? "0"}`;
    case "JmpLabel":
      return `JMPL ${formatLabel(value?.label)}`;
    case "JmpRelReg":
      return `JMRR ${formatRegister(value?.r)}`;
    case "JmpRelImm":
      return `JMRI ${formatNumber(value?.imm) ?? "0"}`;
    case "JLtLabel":
      return `JLTL ${formatRegister(value?.rl)}, ${formatRegister(value?.rr)}, ${formatLabel(
        value?.addr,
      )}`;
    case "JltRel":
      return `JLTI ${formatRegister(value?.rl)}, ${formatRegister(value?.rr)}, ${formatNumber(
        value?.imm,
      ) ?? "0"}`;
    case "JltRelBack":
      return `JLTR ${formatRegister(value?.rl)}, ${formatRegister(value?.rr)}, ${formatNumber(
        value?.imm,
      ) ?? "0"}`;
    default:
      return `[${tag}]`;
  }
}

function formatRegister(raw) {
  const variant = readUnitVariant(raw);
  if (variant) return variant.toLowerCase();
  if (typeof raw === "string") return raw.toLowerCase();
  return "?";
}

function formatLabel(raw) {
  const name = extractLabelName(raw);
  if (!name) return "";
  return name.startsWith(":") ? name : `:${name}`;
}

function extractLabelName(raw) {
  if (typeof raw === "string") return raw;
  if (Array.isArray(raw) && typeof raw[0] === "string") return raw[0];
  if (raw && typeof raw === "object") {
    if (typeof raw[0] === "string") return raw[0];
    if (typeof raw["0"] === "string") return raw["0"];
    const keys = Object.keys(raw);
    if (keys.length === 1 && typeof raw[keys[0]] === "string") {
      return raw[keys[0]];
    }
  }
  return null;
}

function readUnitVariant(raw) {
  if (typeof raw === "string") return raw;
  if (!raw || typeof raw !== "object") return null;
  const keys = Object.keys(raw);
  if (keys.length !== 1) return null;
  return keys[0];
}

function unwrapTupleStruct(raw) {
  if (Array.isArray(raw)) return raw;
  if (raw && typeof raw === "object") {
    if (Array.isArray(raw[0])) return raw[0];
    if (Array.isArray(raw["0"])) return raw["0"];
  }
  return [];
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

function extractVariant(node) {
  if (!node || typeof node !== "object") return null;
  const keys = Object.keys(node);
  if (keys.length !== 1) return null;
  return { tag: keys[0], value: node[keys[0]] };
}

function formatNumber(raw) {
  if (raw == null) return null;
  const big = toBigInt(raw);
  return big == null ? null : big.toString(10);
}

function toIndex(raw, maxLen) {
  const big = toBigInt(raw);
  if (big == null) return undefined;
  if (big < 0n) return undefined;
  if (Number.isFinite(maxLen) && big >= BigInt(maxLen)) return undefined;
  return Number(big);
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
  if (raw && typeof raw === "object" && "Number" in raw) {
    return toBigInt(raw.Number);
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

function safeStringify(value) {
  try {
    return JSON.stringify(value);
  } catch (_) {
    return String(value);
  }
}
